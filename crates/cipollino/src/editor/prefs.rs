
use std::path::PathBuf;

use serde_json::{from_value, json, Map, Value};

use crate::util::fs::{read_json_file, write_json_file};

pub trait UserPref {
    type Type: serde::Serialize + for<'a> serde::Deserialize<'a>;

    fn default() -> Self::Type;
    fn name() -> &'static str;

}

pub struct UserPrefs {
    prefs_path: PathBuf,
    prefs: Map<String, Value>
}

impl UserPrefs {

    pub fn new(prefs_path: PathBuf) -> Self {
        let prefs = if let Some(json) = read_json_file(&prefs_path) {
            if let Some(map) = json.as_object() {
                map.clone()
            } else {
                Map::new()
            }
        } else {
            write_json_file(&prefs_path, json!({}));
            Map::new()
        };

        Self {
            prefs_path,
            prefs
        }
    }

    fn get_existing<P>(&self) -> Option<P::Type> where P: UserPref {
        let val = self.prefs.get(P::name())?;
        let val_parsed = from_value::<P::Type>(val.clone()).ok()?;
        Some(val_parsed)
    }

    pub fn get<P>(&mut self) -> P::Type where P: UserPref {
        if let Some(val) = self.get_existing::<P>() {
            val
        } else {
            self.set::<P>(P::default());
            P::default()
        }
    }

    pub fn set<P>(&mut self, val: P::Type) where P: UserPref {
        self.prefs.insert(P::name().to_owned(), json!(val));
        write_json_file(&self.prefs_path, json!(self.prefs));
    }

}
