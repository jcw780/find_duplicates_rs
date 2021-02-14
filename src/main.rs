use std::collections::{HashMap, HashSet};
use std::fs; 
use std::io::{stdin, stdout, Read};
use std::io::prelude::*;
use std::path::PathBuf;

use base64::encode;
use blake2::{Blake2b, Digest};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "find duplicates", about = "find duplicates.")]
struct Opt{
    #[structopt(parse(from_os_str))]
    target_dir: PathBuf,
}

#[cfg(target_os = "windows")]
fn long_path(path: &PathBuf) -> String{
    #[cfg(target_os = "windows")]
    format!("\\\\?\\{}", path.display())
    // Windows has a 260 character path limit 
    // by default, this overrides it
}

#[cfg(not(target_os = "windows"))]
fn long_path(path: &PathBuf) -> String{
    path.display()
} 

fn get_files_in_dir(target_dir: &PathBuf) -> Vec<PathBuf> {
    let paths = fs::read_dir(target_dir).unwrap();
    paths.map(|path| path.unwrap().path())
        .filter(|path| fs::metadata(path).map_or(false, |v| v.is_file()))
        .collect()
}

fn move_file(src: &PathBuf, dest: &PathBuf){
    fs::rename(long_path(src), long_path(dest)).expect("Failed to move");
}

fn main() {
    let opt = Opt::from_args();
    let files = get_files_in_dir(&opt.target_dir);
    let mut file_hashes = HashMap::<Vec<u8>, Vec<PathBuf>>::new();
    
    println!("Found {} Files", files.len());
    files.iter().for_each(|file| {
        fs::File::open(file).map_or_else(
            |e| println!("Failed to open file: {}\nError: {}", file.display(), e),
            |mut file_input| {
                let mut buffer = [0u8; 32768];
                let mut hasher = Blake2b::new();
                let mut read_success = true;
                while {
                    file_input.read(&mut buffer).map_or_else(
                        |e| {
                            println!("Buffered Read of {} Failed\nError: {}", file.display(), e);
                            read_success = false;
                            false
                        },
                        |n| {
                            hasher.update(buffer);
                            n != 0
                        }
                    )
                }{}; //Do while loop
                if read_success{                
                    let hash: Vec<u8> = hasher.finalize().to_vec();
                    file_hashes.entry(hash)
                        .and_modify(|v| {v.push(file.clone());})
                        .or_insert(vec![file.clone()]);
                }
            },
        );
    });
    let duplicate_folder = {
        let mut temp = opt.target_dir.clone();
        temp.push("duplicates"); temp
    };

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
                move_file(&file.0, &file_dest);
                println!("Moved: {}", &file.1);
            });
        }
    });

    if folders.len() > 0{
        while {
            let mut stdout = stdout();
            stdout.write_all(b"Controls:\nQ: Move remaining back to target_dir\nR: Refresh\n").unwrap();
            stdout.flush().unwrap();

            let mut refresh = true;
            while {
                let mut buffer = String::new();
                let stdin = stdin(); // We get `Stdin` here.
                stdin.read_line(&mut buffer).map_or_else(|_| {
                        println!("Failed to read line");
                        true
                    }, |_| {
                    match &buffer.to_ascii_lowercase().trim()[..]{
                        "r" => {refresh = true; false},
                        "q" => {refresh = false; false},
                        _ => true,
                    }
                })
            }{
                stdout.write_all(b"Incorrect Control Key\n").unwrap();
                stdout.flush().unwrap();
            }
<<<<<<< HEAD
=======

>>>>>>> 9927d073ca9c3477621ea664cd4c5b534edb6f00
            
            if refresh {
                let mut folders_to_remove: HashSet<PathBuf> = HashSet::new();
                folders.iter().for_each(
                    |folder| {
                        let remaining_files = get_files_in_dir(folder);
                        println!("Folder: {}", folder.display());
                        println!("Files Remaining: {}", remaining_files.len());
        
                        if remaining_files.len() < 2 {
                            remaining_files.iter().for_each(|file| {
                                let dest = {
                                    let mut temp = opt.target_dir.clone();
                                    temp.push(file.file_name().unwrap());
                                    temp
                                };
                                move_file(file, &dest);
                            });
                            folders_to_remove.insert(folder.clone());
                        }
                    }
                );
        
                folders_to_remove.iter().for_each(
                    |folder| {
                        fs::remove_dir_all(long_path(folder))
                            .expect("Failed to remove folder");
                    }
                );
                folders.retain(|folder| !(folders_to_remove.contains(folder)));
                folders.len() > 0 //Do while loop
            }else{
                folders.iter().for_each(
                    |folder| {
                        let remaining_files = get_files_in_dir(folder);
                        remaining_files.iter().for_each(|file| {
                            let dest = {
                                let mut temp = opt.target_dir.clone();
                                temp.push(file.file_name().unwrap());
                                temp
                            };
                            move_file(file, &dest);
                        });
                        
                    }
                );
                false
            }
        }{}
        fs::remove_dir_all(duplicate_folder)
            .expect("Failed to remove folder");
    }
}
