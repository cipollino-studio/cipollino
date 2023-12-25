
use super::{Project, ObjData, action::ObjAction};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphicData {
    pub name: String,
    pub len: u32,
    pub clip: bool,
    pub w: u32,
    pub h: u32
}

pub struct Graphic {
    pub data: GraphicData,
    pub layers: Vec<u64>
}

impl ObjData for GraphicData {

    fn add(&self, key: u64, project: &mut Project) {
        project.add_graphic_with_key(key, self.clone());
    }

    fn delete(&self, key: u64, project: &mut Project) {
        project.delete_graphic(key);
    }

    fn set(&self, key: u64, project: &mut Project) {
        project.set_graphic_data(key, self.clone());
    }

}

impl Project {

    pub fn add_graphic(&mut self, data: GraphicData) -> (u64, ObjAction) {
        let key = self.next_key();
        (key, self.add_graphic_with_key(key, data))
    }

    pub fn add_graphic_with_key(&mut self, key: u64, data: GraphicData) -> ObjAction {
        self.graphics.insert(key, Graphic {
            data: data.clone(), 
            layers: Vec::new()
        });
        ObjAction::addition(key, data)
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
