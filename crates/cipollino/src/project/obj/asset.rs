
use crate::project::Project;

use super::{Obj, ObjBox};

pub trait Asset : Obj {

    fn name(&self) -> &String;

}

pub fn next_valid_name<T: Asset>(project: &Project, name: String, assets: &Vec<ObjBox<T>>) -> String {
    let names = assets.iter().map(|asset| asset.get(project).name());

    if names.clone().position(|other_name| other_name == &name).is_none() {
        return name;
    }

    for i in 1.. {
        let potential_name = format!("{} ({})", name, i);
        if names.clone().position(|other_name| other_name == &potential_name).is_none() {
            return potential_name;
        }
    }

    "".to_owned()
}
