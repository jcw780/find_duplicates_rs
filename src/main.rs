use std::collections::{HashMap, HashSet};
use std::fs; 
use std::io::{stdin, stdout, Read};
use std::io::prelude::*;
use std::path::PathBuf;

use base64::encode;
use blake3::{Hasher};
use structopt::StructOpt;

mod file_utilities;

#[derive(Debug, StructOpt)]
#[structopt(name = "find duplicates", about = "find duplicates.")]
struct Opt{
    #[structopt(parse(from_os_str))]
    target_dir: PathBuf,

    #[structopt(short = "d", long = "duplicate", default_value ="duplicates")]
    duplicate_dir: PathBuf,
}

fn loop_till_valid_key<F>(prompt: &[u8], mut key_check: F) where F: FnMut(&str) -> bool{
    let mut stdout = stdout();
    stdout.write_all(prompt).unwrap();
    stdout.flush().unwrap();
    while {
        let mut buffer = String::new();
        let stdin = stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).map_or_else(|_| {
                println!("Failed to read line");
                true
            }, |_| {
                key_check(&buffer.to_ascii_lowercase().trim()[..])
            }
        )
    }{
        stdout.write_all(b"Incorrect Control Key\n").unwrap();
        stdout.flush().unwrap();
    }
}

fn group_file_paths_by_hash(files: &Vec<PathBuf>) -> HashMap<[u8; 32], Vec<PathBuf>>{
    let mut file_hashes = HashMap::<[u8; 32], Vec<PathBuf>>::new();
    files.iter().for_each(|file| {
        fs::File::open(file).map_or_else(
            |e| println!("Failed to open file: {}\nError: {}", file.display(), e),
            |mut file_input| {
                let mut buffer = [0u8; 32768];
                let mut hasher = Hasher::new();
                let mut read_success = true;
                while {
                    file_input.read(&mut buffer).map_or_else(
                        |e| {
                            println!("Buffered Read of {} Failed\nError: {}", file.display(), e);
                            read_success = false;
                            false
                        },
                        |n| {
                            hasher.update(&buffer);
                            n != 0
                        }
                    )
                }{}; //Do while loop
                if read_success{                
                    let hash: [u8; 32] = *hasher.finalize().as_bytes();
                    file_hashes.entry(hash)
                        .and_modify(|v| {v.push(file.clone());})
                        .or_insert(vec![file.clone()]);
                }
            },
        );
    });
    file_hashes
}

fn view_move_duplicates(file_hashes: &HashMap<[u8; 32], Vec<PathBuf>>, target_dir: &PathBuf, duplicate_folder: &PathBuf){
    let mut folders: Vec<PathBuf> = vec![];
    file_hashes.iter().for_each(|(hash, files)| {
        if files.len() > 1{
            let hash_b64 = encode(hash).replace("/", "_").replace("+", "-");
            let hash_dir = {
                let mut temp = duplicate_folder.clone();
                temp.push(&hash_b64); temp
            };
            fs::create_dir_all(&hash_dir).expect("Failed to create folder");
            folders.push(hash_dir.clone());
            let file_names: Vec<(PathBuf, String)> = files.iter()
                .map(|file| {
                    (
                        file.clone(), 
                        file.file_name().expect("Path is not a proper file name")
                            .to_str().expect("Path is not valid in UTF-8").to_string()
                    )
                }).collect();
            println!("Hash: {}", hash_b64);
            file_names.iter().for_each(|file| {
                let file_dest = {
                   let mut temp = hash_dir.clone();
                   temp.push(&file.1); temp
                }; 
                file_utilities::move_file(&file.0, &file_dest);
                println!("Moved: {}", &file.1);
            });
        }
    });

    let move_remaining_files = |remaining_files: Vec<PathBuf>|{
        remaining_files.iter().for_each(|file| {
            let dest = {
                let mut temp = target_dir.clone();
                temp.push(file.file_name().unwrap());
                temp
            };
            file_utilities::move_file(file, &dest);
        });
    };

    if folders.len() > 0{
        while {
            let mut stdout = stdout();
            stdout.write_all(b"Controls:\nQ: Move remaining back to target_dir\nR: Refresh\n").unwrap();
            stdout.flush().unwrap();

            let mut refresh = true;
            loop_till_valid_key(b"Controls:\nQ: Move remaining back to target_dir\nR: Refresh\n", |key: &str| {
                match key{
                    "r" => {refresh = true; false},
                    "q" => {refresh = false; false},
                    _ => true,
                }
            });            
            if refresh {
                let mut folders_to_remove: HashSet<PathBuf> = HashSet::new();
                folders.iter().for_each(
                    |folder| {
                        let remaining_files = file_utilities::get_files_in_dir(folder);
                        println!("Folder: {}", folder.display());
                        println!("Files Remaining: {}", remaining_files.len());
        
                        if remaining_files.len() < 2 {
                            move_remaining_files(remaining_files);
                            folders_to_remove.insert(folder.clone());
                        }
                    }
                );
        
                folders_to_remove.iter().for_each(
                    |folder| {
                        fs::remove_dir_all(file_utilities::long_path(folder))
                            .expect("Failed to remove folder");
                    }
                );
                folders.retain(|folder| !(folders_to_remove.contains(folder)));
                folders.len() > 0 //Do while loop
            }else{
                folders.iter().for_each(
                    |folder| {
                        let remaining_files = file_utilities::get_files_in_dir(folder);
                        move_remaining_files(remaining_files);
                    }
                );
                false
            }
        }{} //Do while loop
        fs::remove_dir_all(duplicate_folder)
            .expect("Failed to remove folder");
    }
}

fn main() {
    let opt = Opt::from_args();
    let target_dir = opt.target_dir;
    let duplicate_folder = {
        let mut temp = target_dir.clone();
        temp.push(opt.duplicate_dir.clone()); temp
    };

    let mut run = true;
    if duplicate_folder.exists() {
        loop_till_valid_key(b"Duplicates directory exists, further operation may overwrite internal data\nProceed? Y/N\n", 
            |key: &str|{
                match key{
                    "y" => {run = true; false},
                    "n" => {run = false; false},
                    _ => true,
                }
            }
        );
    }

    if run {
        let files_in_target = file_utilities::get_files_in_dir(&target_dir);
        println!("Found {} Files", files_in_target.len());
        let file_hashes = group_file_paths_by_hash(&files_in_target);
        view_move_duplicates(&file_hashes, &target_dir, &duplicate_folder);
    }
}
