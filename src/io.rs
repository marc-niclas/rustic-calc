use std::{env, fs, io::Error, path::PathBuf};

use crate::types::AppState;

pub fn create_rcalc_dir() -> Result<(), std::io::Error> {
    fs::create_dir_all(get_config_dir()?)?;
    Ok(())
}

pub fn write_state_to_file(app: &AppState) -> Result<(), std::io::Error> {
    create_rcalc_dir()?;
    let json = serde_json::to_string(app).map_err(Error::other)?;
    fs::write(get_state_file_path()?, json)?;
    Ok(())
}

pub fn get_state_from_file() -> Result<AppState, std::io::Error> {
    let data = fs::read_to_string(get_state_file_path()?)?;
    let state = serde_json::from_str(&data).map_err(Error::other)?;
    Ok(state)
}

pub fn reset_file_state() -> Result<(), std::io::Error> {
    let file_path = get_state_file_path()?;
    fs::remove_file(file_path)?;
    create_rcalc_dir()?;
    Ok(())
}

fn get_config_dir() -> Result<PathBuf, std::io::Error> {
    match env::var("HOME") {
        Ok(home) => Ok(PathBuf::from(home).join(".config").join("rcalc")),
        Err(err) => Err(Error::other(err)),
    }
}

fn get_state_file_path() -> Result<PathBuf, std::io::Error> {
    Ok(get_config_dir()?.join("state.json"))
}
