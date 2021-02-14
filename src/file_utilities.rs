use std::fs; 
use std::path::PathBuf;

#[cfg(target_os = "windows")]
pub fn long_path(path: &PathBuf) -> String{
    format!("\\\\?\\{}", path.display())
    // Windows has a 260 character path limit 
    // by default, this overrides it
}

#[cfg(not(target_os = "windows"))]
pub fn long_path(path: &PathBuf) -> String{
    path.display()
} 

pub fn get_files_in_dir(target_dir: &PathBuf) -> Vec<PathBuf> {
    let paths = fs::read_dir(target_dir).unwrap();
    paths.map(|path| path.unwrap().path())
        .filter(|path| fs::metadata(path).map_or(false, |v| v.is_file()))
        .collect()
}

pub fn move_file(src: &PathBuf, dest: &PathBuf){
    fs::rename(long_path(src), long_path(dest)).expect("Failed to move");
}