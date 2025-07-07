use crate::for_debug;
use cli_log::Metadata;
use ratatui::widgets::{ListItem, ListState};
use size::Size;
use std::{
    fs::{self, File},
    io, path,
};
use tracing::debug;

#[derive(Debug)]
pub struct FileItem {
    name: String,
    metadata: fs::Metadata,
    size: Size,
    is_dir: bool,
}

impl FileItem {
    //The str that is returned cant outlive where the FileItem is living
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
    pub fn size(&self) -> String {
        let textual = format!("{}", self.size);
        textual.clone()
    }
}

pub struct FileList {
    pub files: Vec<FileItem>,
    pub state: ListState,
}

impl FileList {
    pub fn from_path(path_str: &String) -> Self {
        let files = list_dir(path_str).unwrap();
        let state = ListState::default();
        Self { files, state }
    }
    /*
    pub fn to_formated_list_item(&self) -> Vec<ListItem> {
        self.files
            .iter()
            .map(|file| {
                ListItem::from(format!(
                    "{:}                              {:}",
                    file.name().clone(),
                    file.size().clone()
                ))
            })
            .collect()
    }
    */
}

fn get_file_size(file_metadata: &std::fs::Metadata, path: String, mut total_size: u64) -> u64 {
    if file_metadata.is_dir() {
        for entry_result in fs::read_dir(&path).unwrap() {
            if let Ok(entry) = entry_result {
                let file_name = entry.file_name().into_string().unwrap();
                let inner_file_result = File::open(path.clone() + "/" + &file_name);
                let inner_file_metadata: std::fs::Metadata;

                //The loop can omit files that causes error like :"No such device or address"
                //because i am too lazy to solve all of them
                if let Ok(inner_file) = inner_file_result {
                    inner_file_metadata = inner_file.metadata().unwrap();
                } else {
                    continue;
                }
                /*
                let format = format!(
                    "{} : {}",
                    path.clone() + "/" + &file_name,
                    &total_size.to_string()
                );
                debug!(format);
                */
                total_size = get_file_size(
                    &inner_file_metadata,
                    path.clone() + "/" + &file_name,
                    total_size,
                );
            } else {
                println!(
                    "Error reading a entry from the directory: {:?}",
                    entry_result
                );
            }
        }
        total_size
    } else {
        total_size + file_metadata.len()
    }
}

pub fn list_dir(str_path: &String) -> Result<Vec<FileItem>, io::Error> {
    let dir_path = path::PathBuf::from(str_path.clone());
    let mut files: Vec<FileItem> = Vec::new();
    for entry_result in fs::read_dir(dir_path)? {
        //I guess is used .as_ref() because from the DirEntry i only need the name? so there is no need for
        //owning
        if let Ok(entry) = entry_result {
            // entry.file_name().to_str().unwrap() , doesnt work because entry.file_name() generates a owned
            // os_str, that then is referenced with to_str, but how i didnt saved the os_str
            // anywhere, it is dropped and the reference have nothing to point to

            let file_name = entry.file_name().to_string_lossy().into_owned();
            // File::open has to take in account the current path
            //
            let path_and_inner_file = str_path.clone() + &file_name.clone();
            let file_instance = File::open(path_and_inner_file.clone())?;
            let file_metadata = file_instance.metadata()?;
            let mut file_size: u64 = 0; // in bytes
            file_size = get_file_size(&file_metadata, path_and_inner_file.clone(), file_size);
            files.push(FileItem {
                name: file_name,
                metadata: file_metadata,
                size: Size::from_bytes(file_size),
                is_dir: file_instance.metadata().unwrap().is_dir(),
            });
        } else {
            println!(
                "Error reading a entry from the directory: {:?}",
                entry_result
            );
        }
    }
    Ok(files)
}
