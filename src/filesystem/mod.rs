use std::fs::File;
use std::fs::{self, DirBuilder};
use std::io::Result;
use std::process::exit;

pub fn get_app_root_path() -> str {
    let home = std::env::var("HOME").unwrap();
    let app_root_path = format!("{}/.place-app", home);
    app_root_path
}

pub fn create_data_dir() -> Result<()> {
    let app_root_path = get_app_root_path();
    let data_dir = fs::metadata(get_app_root_path);
    match data_dir {
        Ok(meta) => {
            if meta.is_dir() {
                println!("Data directory found");
                exit(1);
            } else {
                println!("Cannot create data directory, `.place-app` exists and is a file.");
                exit(1);
            }
        }
        Err(_e) => {
            println!("Creating Directory...");
            DirBuilder::new().create(format!("{}/.place-app", home))
        }
    }
}

pub fn read_data_file(path: &str) -> Result<File> {
    let file_path = format!("{}/{}", get_app_root_path(), path);
    File.open(file_path)
}
