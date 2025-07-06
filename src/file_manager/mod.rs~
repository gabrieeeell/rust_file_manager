use size::Size;
use std::{
    fs::{self, DirEntry, File},
    io, path,
};

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
}

fn main() {
    let mut curr_path = String::from("./");
    loop {
        process_curr_path(&curr_path);
        curr_path.clear();
        io::stdin().read_line(&mut curr_path);
    }
}

//true if the path is a dir
fn process_curr_path(path: &String) -> std::io::Result<()> {
    let file = File::open(&path).unwrap();
    let file_metadata = file.metadata().unwrap();
    // i have to use the .file_type() method not the .file_type atributte because it is private
    if file_metadata.file_type().is_dir() {
        list_dir(path);
    } else {
    }

    Ok(())
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
            let path_and_inner_file = str_path.clone() + "/" + &file_name.clone();
            let file_instance = File::open(path_and_inner_file)?;
            let file_metadata = file_instance.metadata()?;
            let file_size = Size::from_bytes(file_metadata.len());
            files.push(FileItem {
                name: file_name,
                metadata: file_metadata,
                size: file_size,
                is_dir: file_instance.metadata().unwrap().is_dir(),
            });
        } else {
            println!(
                "Error reading a entry from the directory: {:?}",
                entry_result
            );
        }
    }
    /*
        for file in files {
            if file.is_dir {
                println!(
                    "/{:}                             {:?}",
                    file.name, file.size
                );
            } else {
                println!("{:}                             {:?}", file.name, file.size);
            }
        }
    */
    Ok(files)
}
