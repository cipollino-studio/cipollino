
pub mod curve;
pub mod geo;
pub mod ui;
pub mod color;
pub mod fs;
pub mod bson;
pub mod ffmpeg;

pub fn next_unique_name<'a, T>(name: &String, names: T) -> String where T: Iterator<Item = &'a str> + Clone {
    if names.clone().position(|other_name| other_name.to_lowercase() == name.to_lowercase()).is_none() {
        return name.clone();
    }

    for i in 1.. {
        let potential_name = format!("{} ({})", name, i);
        if names.clone().position(|other_name| other_name.to_lowercase() == potential_name.to_lowercase()).is_none() {
            return potential_name;
        }
    }

    "".to_owned()
}
