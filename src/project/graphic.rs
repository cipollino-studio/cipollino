
use super::{Project, ObjData, action::ObjAction};

#[derive(Clone)]
pub struct GraphicData {
    pub name: String,
    pub len: u32,
}

pub struct Graphic {
    pub data: GraphicData,
    pub layers: Vec<u64>
}

impl ObjData for GraphicData {

    fn add(&self, key: u64, project: &mut Project) {
        project.add_graphic_with_key(key, self.name.clone(), self.len);
    }

    fn delete(&self, key: u64, project: &mut Project) {
        project.delete_graphic(key);
    }

    fn set(&self, key: u64, project: &mut Project) {
        project.set_graphic_data(key, self.clone());
    }

}

impl Project {

    pub fn add_graphic(&mut self, name: String, len: u32) -> (u64, ObjAction) {
        let key = self.next_key();
        (key, self.add_graphic_with_key(key, name, len))
    }

    pub fn add_graphic_with_key(&mut self, key: u64, name: String, len: u32) -> ObjAction {
        let graphic_data = GraphicData {
            name,
            len
        };
        self.graphics.insert(key, Graphic {
            data: graphic_data.clone(),
            layers: Vec::new()
        });
        ObjAction::addition(key, graphic_data)
    }

    pub fn delete_graphic(&mut self, key: u64) -> Option<()> {
        let graphic = self.graphics.remove(&key)?;
        for layer_key in graphic.layers {
            self.delete_layer(layer_key);
        }
        None
    }

    pub fn set_graphic_data(&mut self, key: u64, data: GraphicData) -> Option<ObjAction> {
        let graphic = self.graphics.get_mut(&key)?;
        let res = ObjAction::modification(key, graphic.data.clone(), data.clone());
        graphic.data = data;
        Some(res)
    }

}
