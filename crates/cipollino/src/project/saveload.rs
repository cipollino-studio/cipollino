
use std::{fs, io::{Read, Write}, path::PathBuf};

use serde_json::json;

use crate::project::{graphic::Graphic, obj::ObjBox};

use super::{obj::{asset::Asset, ObjClone}, Project};

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
        for gfx_box in &self.root_graphics {
            let gfx = gfx_box.get(self);
            let data = gfx_box.obj_serialize(self);
            write_json_file(&folder_path.with_file_name(format!("{}.cipgfx", gfx.name())), data);
        }
        
        let data = json!{{
            "curr_keys": {
                "graphics": self.graphics.curr_key,
                "layers": self.layers.curr_key,
                "frames": self.frames.curr_key,
                "strokes": self.strokes.curr_key,
            }
        }};
        write_json_file(&folder_path.with_file_name("proj.cip"), data);

        self.save_path = Some(folder_path.with_file_name("proj.cip"));
    }

    pub fn load(proj_file_path: PathBuf) -> Self {
        let mut res = Self::new();

        if let Some(proj_data) = read_json_file(&proj_file_path) {
            let mut read_curr_keys = || {
                let curr_keys = proj_data.get("curr_keys")?.as_object()?;
                res.graphics.curr_key = curr_keys.get("graphics").unwrap_or(&json!(1)).as_u64().unwrap_or(1);
                res.layers.curr_key = curr_keys.get("layers").unwrap_or(&json!(1)).as_u64().unwrap_or(1);
                res.frames.curr_key = curr_keys.get("frames").unwrap_or(&json!(1)).as_u64().unwrap_or(1);
                res.strokes.curr_key = curr_keys.get("strokes").unwrap_or(&json!(1)).as_u64().unwrap_or(1);
                Some(()) 
            };

            read_curr_keys();
        }

        let folder_path = proj_file_path.parent().unwrap();
        if let Ok(paths) = fs::read_dir(folder_path) {
            for path in paths {
                if let Ok(path) = path {
                    if let Some(ext) = path.path().extension() {
                        if ext == "cipgfx" {
                            if let Some(data) = read_json_file(&path.path()) { 
                                if let Some(gfx) = ObjBox::<Graphic>::obj_deserialize(&mut res, &data) {
                                    res.root_graphics.push(gfx);
                                }
                            }
                        }
                    } 
                }
            }
        }

        res.save_path = Some(proj_file_path);

        res
    }

}
