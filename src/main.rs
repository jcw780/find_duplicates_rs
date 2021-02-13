use std::collections::HashMap;
use std::fs; 
use std::io::Read;
use std::path::PathBuf;

use blake2::{Blake2b, Digest};
use hex;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "find duplicates", about = "find duplicates.")]
struct Opt{
    #[structopt(parse(from_os_str))]
    target_dir: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let paths = fs::read_dir(&opt.target_dir).unwrap();
    let mut file_hashes = HashMap::<Vec<u8>, Vec<PathBuf>>::new();

    let files: Vec<PathBuf> = paths.map(|path| path.unwrap().path())
        .filter(|path| fs::metadata(path).map_or(false, |v| v.is_file()))
        .collect();
    
    println!("Found {} Files", files.len());
    files.iter().for_each(|file| {
        match fs::File::open(file){
            Ok(mut file_input) => {
                let mut buffer = [0u8; 32768];
                let mut hasher = Blake2b::new();
                loop {
                    match file_input.read(&mut buffer){
                        Ok(n) => {
                            if n == 0 {break}
                            hasher.update(buffer);
                        },
                        Err(e) => {
                            println!("Buffered Read of {} Failed\nError: {}", file.display(), e);
                            break;
                        }
                    }
                } 
                let hash: Vec<u8> = hasher.finalize().to_vec();
                file_hashes.entry(hash)
                    .and_modify(|v| {v.push(file.clone());})
                    .or_insert(vec![file.clone()]);
            },
            Err(e) => println!("Failed to open file: {}\nError: {}", file.display(), e)
        }
    });

    println!("Duplicates: ");
    file_hashes.iter().for_each(|(hash, files)| {
        if files.len() > 1{
            let hash_hex = hex::encode(hash);
            let duplicate_dir = format!("{}/duplicates/{}", &opt.target_dir.display(), hash_hex);
            fs::create_dir_all(&duplicate_dir).expect_err("Failed to create folder");
            let file_names: Vec<(PathBuf, String)> = files.iter()
                .map(|file| {
                    file.file_name().map_or((file.clone(), "".to_string()), |name_os| {
                        name_os.to_str().map_or((file.clone(), "".to_string()), |name| {
                            (file.clone(), name.to_string())
                        })
                    })
                }).collect();
            println!("Hash: {}", hash_hex);
            file_names.iter().for_each(|file| {
                println!("Moved: {}", &file.1);
                match fs::copy(&file.0, format!("{}/{}", &duplicate_dir, &file.1)){
                    Ok(_r) => (), Err(e) => println!("Failed to copy {} to folder\nError: {}", &file.1, e),
                }
                match fs::remove_file(&file.0){
                    Ok(_r) => (), Err(e) => println!("Failed to remove {}\nError: {}", &file.1, e),
                }
            });
        }
    });
}
