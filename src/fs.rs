use std::{fs::create_dir_all, path::PathBuf};

use anyhow::bail;
use directories::ProjectDirs;

pub fn get_data_directory(path: Option<&str>) -> anyhow::Result<PathBuf> {
    // TODO: Fix the namespace
    let Some(project_directories) = ProjectDirs::from("com", "s9tpepper", "FerrisTwitch") else {
        bail!("Could not get data directory")
    };

    let mut data_directory = project_directories.data_dir().to_path_buf();

    //data_directory.push("v2");

    if let Some(path) = path {
        data_directory.push(path);
    }

    if !data_directory.exists() {
        create_dir_all(&data_directory)?;
    }

    Ok(data_directory)
}
