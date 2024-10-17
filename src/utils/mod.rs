use descape::UnescapeExt;
use std::{error::Error, fs, path::PathBuf};

use directories::ProjectDirs;

use crate::commands::TokenStatus;

pub fn read_auth_token() -> core::result::Result<TokenStatus, Box<dyn Error>> {
    let error_message =
        "You need to provide credentials via positional args, env vars, or by running the login command";
    let mut data_dir = get_data_directory(Some("token")).expect(error_message);
    data_dir.push("oath_token.txt");

    let token_file = fs::read_to_string(data_dir)?;

    Ok(serde_json::from_str::<TokenStatus>(&token_file)?)
}

pub fn get_data_directory(path: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(project_directories) = ProjectDirs::from("com", "s9tpepper", "FerrisTwitch") {
        let mut data_directory = project_directories.data_dir().to_path_buf();
        if let Some(path) = path {
            data_directory.push(path);
        }

        if !data_directory.exists() {
            std::fs::create_dir_all(&data_directory)?;
        }

        return Ok(data_directory);
    }

    Err("Could not get data directory".into())
}

pub fn unescape(escaped_str: &str) -> String {
    let unescape_result = escaped_str.to_unescaped();
    if let Ok(unescaped_str) = unescape_result {
        return unescaped_str.replace(r"\s", " ");
    }

    escaped_str.to_string().replace(r"\s", " ")
}

#[test]
fn test_unescape() {
    let test_string = r"7\\sraiders\\sfrom\\sMatisseTec\\shave\\sjoined!";

    assert_eq!(unescape(test_string), r"7 raiders from MatisseTec have joined!");
}

#[test]
fn test_unescape_2() {
    let test_string = r"1\sraiders\sfrom\svei_bean\shave\sjoined!";

    assert_eq!(unescape(test_string), r"1 raiders from vei_bean have joined!");
}
