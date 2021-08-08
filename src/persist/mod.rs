#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use typings::fixed::Statics;

mod ensure_player_locations;
pub mod player;
pub mod site;

fn read<P: AsRef<Path>, T>(file: P) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let value = serde_yaml::from_str(&fs::read_to_string(file)?)?;
    Ok(value)
}

fn write<P: AsRef<Path>, T>(file: P, value: &T) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    write_str(file.as_ref(), &serde_yaml::to_string(value)?)?;
    Ok(())
}

fn write_str(file: &Path, new_content: &str) -> std::io::Result<()> {
    if fs::read_to_string(file).map_or(true, |current| current != new_content) {
        fs::create_dir_all(file.parent().unwrap())?;
        fs::write(file, new_content)?;
    }
    Ok(())
}

fn delete(filename: &str) -> std::io::Result<()> {
    if Path::new(filename).exists() {
        fs::remove_file(filename)?;
    }
    Ok(())
}

fn list<P: AsRef<Path>>(folder: P) -> Vec<PathBuf> {
    let folder = folder.as_ref();
    let mut result = Vec::new();
    for entry in fs::read_dir(folder)
        .expect("failed to list files")
        .filter_map(|o| o.ok())
    {
        if entry.path().is_dir() {
            let mut children = list(entry.path());
            result.append(&mut children);
        } else {
            result.push(entry.path().clone());
        }
    }
    result
}

pub fn init(statics: &Statics) -> anyhow::Result<()> {
    site::ensure_statics(&statics.solarsystems)?;
    ensure_player_locations::ensure_player_locations(statics)?;
    Ok(())
}
