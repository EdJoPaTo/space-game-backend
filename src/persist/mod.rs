#![allow(dead_code)]

use std::fs;
use std::path::Path;

use typings::fixed::Statics;

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

pub fn init(statics: &Statics) -> anyhow::Result<()> {
    site::ensure_statics(&statics.solarsystems)?;
    Ok(())
}
