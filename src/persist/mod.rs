#![allow(clippy::module_name_repetitions)]

use std::fs;
use std::path::{Path, PathBuf};

mod ensure_player_locations;
pub mod player;
pub mod site;

pub use ensure_player_locations::ensure_player_locations;
pub use site::ensure_static_sites;

fn read<P: AsRef<Path>, T>(file: P) -> T
where
    T: serde::de::DeserializeOwned + Default,
{
    let file = file.as_ref();
    if let Ok(content) = fs::read_to_string(file) {
        match serde_yaml::from_str(&content) {
            Ok(result) => result,
            Err(err) => panic!("failed to deserialize {:?} {}", file, err),
        }
    } else {
        T::default()
    }
}

fn read_meh<P: AsRef<Path>, T>(file: P) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let file = file.as_ref();
    let content = fs::read_to_string(file)
        .map_err(|err| anyhow::anyhow!("failed to read {:?} {}", file, err))?;
    let value = serde_yaml::from_str(&content)
        .map_err(|err| anyhow::anyhow!("failed to deserialize {:?} {}", file, err))?;
    Ok(value)
}

fn write<P: AsRef<Path>, T>(file: P, value: &T) -> anyhow::Result<()>
where
    T: serde::Serialize + Default + std::cmp::PartialEq,
{
    if value == &T::default() {
        delete(file)?;
    } else {
        let file = file.as_ref();
        let content = serde_yaml::to_string(value)
            .map_err(|err| anyhow::anyhow!("failed to serialize {:?} {}", file, err))?;
        write_str(file, &content)
            .map_err(|err| anyhow::anyhow!("failed to write {:?} {}", file, err))?;
    }
    Ok(())
}

fn write_str(file: &Path, new_content: &str) -> std::io::Result<()> {
    if fs::read_to_string(file).map_or(true, |current| current != new_content) {
        fs::create_dir_all(file.parent().unwrap())?;
        fs::write(file, new_content)?;
    }
    Ok(())
}

fn delete<P: AsRef<Path>>(file: P) -> std::io::Result<()> {
    let file = file.as_ref();
    if file.exists() {
        fs::remove_file(file)?;
    }
    Ok(())
}

fn list<P: AsRef<Path>>(folder: P) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(direntry) = fs::read_dir(folder) {
        for entry in direntry.filter_map(|o| o.ok()) {
            if entry.path().is_dir() {
                let mut children = list(entry.path());
                result.append(&mut children);
            } else {
                result.push(entry.path().clone());
            }
        }
    }
    result
}
