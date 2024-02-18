use std::{path::PathBuf, error::Error};

use directories::ProjectDirs;

pub fn get_data_directory() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(project_directories) = ProjectDirs::from("com", "s9tpepper", "FerrisTwitch") {
        let data_directory = project_directories.data_dir();

        if !data_directory.exists() {
            std::fs::create_dir_all(data_directory)?;
        }

        return Ok(data_directory.to_path_buf())
    }

    Err("Could not get data directory".into())
}
