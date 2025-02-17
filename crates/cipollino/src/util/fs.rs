use std::{fs, io::{Read, Write}, path::PathBuf};


pub fn read_json_file(path: &PathBuf) -> Option<serde_json::Value> {
    let mut file = fs::File::open(path).ok()?;
    let mut str = "".to_owned();
    file.read_to_string(&mut str).ok()?;
    serde_json::from_str::<serde_json::Value>(str.as_str()).ok()
}

pub fn write_json_file(path: &PathBuf, data: serde_json::Value) -> Option<()> {
    let str = data.to_string();
    let mut file = fs::File::create(path).ok()?;
    file.write(str.as_bytes()).ok()?;
    Some(())
}

pub fn read_bson_file(path: &PathBuf) -> Option<bson::Bson> {
    let file = fs::File::open(path).ok()?;
    bson::from_reader(file).ok()
}

pub fn write_bson_file(path: &PathBuf, data: bson::Bson) -> Option<()> {
    let file = fs::File::create(path).ok()?;
    let doc = data.as_document().expect("cannot serialize non-document");
    doc.to_writer(file).ok()?;
    Some(())
}

pub fn remove(path: &PathBuf) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else {
        let _ = fs::remove_file(path);
    }
}

pub fn set_file_stem(path: &mut PathBuf, stem: &str) {
    if let Some(ext) = path.extension() {
        path.set_file_name(format!("{}.{}", stem, ext.to_str().unwrap()));
    } else {
        path.set_file_name(stem);
    }
}

#[cfg(target_os = "macos")]
pub fn trash_folder() -> PathBuf {
    directories::UserDirs::new().unwrap().home_dir().join(".Trash")
}

#[cfg(target_os = "windows")]
pub fn trash_folder() -> PathBuf {
    PathBuf::from("C:/$Recycle.Bin")
}
