#![allow(dead_code)]

use std::path::Path;

use anyhow::Result;
use typings::fixed::Statics;

pub mod player;
pub mod site;

fn read<T>(filename: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let value = serde_yaml::from_str::<T>(&std::fs::read_to_string(filename)?)?;
    Ok(value)
}

fn write<T>(filename: &str, value: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let path = Path::new(filename);
    let folder = path.parent().unwrap();
    if !folder.exists() {
        std::fs::create_dir_all(folder)?;
    }

    std::fs::write(path, serde_yaml::to_string(value)?)?;
    Ok(())
}

fn delete(filename: &str) -> Result<()> {
    let path = Path::new(filename);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn ensure_statics(statics: &Statics) -> Result<()> {
    site::ensure_statics(&statics.solarsystems)?;
    Ok(())
}
