
use std::{fs, io::{Read, Write}, path::PathBuf};

use serde_json::json;

use crate::project::{graphic::Graphic, obj::ObjBox};

use super::{folder::Folder, obj::{asset::Asset, ObjPtr, ObjSerialize}, palette::Palette, Project};

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


impl Project {

    pub fn save(&mut self, mut folder_path: PathBuf) {
        if folder_path.is_dir() {
            folder_path.push("a");
        }

        for file in &self.files_to_delete {
            let path = folder_path.with_file_name(file.clone());
            if path.is_dir() {
                let _ = fs::remove_dir_all(path);
            } else {
                let _ = fs::remove_file(path);
            }
        }
        self.files_to_delete.clear();

        self.save_folder(self.root_folder.make_ptr(), &folder_path);
        
        write_json_file(&folder_path.with_file_name("proj.cip"), json!({}));

        self.save_path = Some(folder_path.with_file_name("proj.cip"));
    }

    pub fn save_folder(&mut self, folder: ObjPtr<Folder>, path: &PathBuf) {
        let mut subfolders = Vec::new();
        if let Some(folder) = self.folders.get(folder) {
            for subfolder in &folder.folders {
                let mut new_path = path.with_file_name(subfolder.get(self).name.clone()); 
                let _ = std::fs::create_dir(new_path.clone());
                new_path.push("a");
                subfolders.push((subfolder.make_ptr(), new_path));
            }
            for gfx_box in &folder.graphics {
                let gfx = gfx_box.get(self);
                let data = gfx_box.obj_serialize(self);
                write_json_file(&path.with_file_name(format!("{}.{}", gfx.name, gfx.extension())), data);
            }
            for palette_box in &folder.palettes {
                let palette = palette_box.get(self);
                let data = palette_box.obj_serialize(self);
                write_json_file(&path.with_file_name(format!("{}.{}", palette.name(), palette.extension())), data);
            }
        }
        for (ptr, path) in subfolders {
            self.save_folder(ptr, &path);
        }
    }

    pub fn load(proj_file_path: PathBuf) -> Self {
        let mut res = Self::new();

        if let Some(_proj_data) = read_json_file(&proj_file_path) {
            
        }

        let folder_path = proj_file_path.parent().unwrap();
        res.save_path = Some(proj_file_path.clone());
        res.root_folder = res.load_folder(&folder_path.to_owned(), ObjPtr::null()); 

        res
    }

    fn load_folder(&mut self, path: &PathBuf, parent: ObjPtr<Folder>) -> ObjBox<Folder> {
        let res = self.folders.add(Folder::new(parent));
        res.get_mut(self).name = path.file_name().unwrap().to_str().unwrap().to_owned();

        if let Ok(paths) = fs::read_dir(path) {
            for path in paths {
                if let Ok(path) = path {
                    let path = path.path();
                    if let Some(ext) = path.extension() {
                        if ext == "cipgfx" {
                            if let Some(data) = read_json_file(&path) { 
                                if let Some(gfx) = ObjBox::<Graphic>::obj_deserialize(self, &data, res.make_ptr().into()) {
                                    gfx.get_mut(self).name = path.file_stem().unwrap().to_str().unwrap().to_owned();
                                    res.get_mut(self).graphics.push(gfx);
                                }
                            }
                        }
                        if ext == "cippal" {
                            if let Some(data) = read_json_file(&path) { 
                                if let Some(palette) = ObjBox::<Palette>::obj_deserialize(self, &data, res.make_ptr().into()) {
                                    let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
                                    palette.get_mut(self).name = name; 
                                    res.get_mut(self).palettes.push(palette);
                                }
                            }
                        }
                    } 
                    if path.is_dir() {
                        let folder = self.load_folder(&path, res.make_ptr());
                        res.get_mut(self).folders.push(folder);
                    }
                }
            }
        }
        res
    } 

}
