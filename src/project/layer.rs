
use super::{Project, action::ObjAction, ObjData};

#[derive(Clone)]
pub struct LayerData {
    pub gfx: u64,
    pub name: String
}

impl ObjData for LayerData {

    fn add(&self, key: u64, project: &mut Project) {
        project.add_layer_with_key(key, self.gfx, self.name.clone());
    }

    fn delete(&self, key: u64, project: &mut Project) {
        project.delete_layer(key);
    }

    fn set(&self, key: u64, project: &mut Project) {
        project.set_layer_data(key, self.clone());
    }

}

pub struct Layer {
    pub data: LayerData
}

impl Project {

    pub fn add_layer(&mut self, gfx: u64, name: String) -> Option<(u64, ObjAction)> {
        let key = self.next_key();
        self.add_layer_with_key(key, gfx, name)
    }
    
    pub fn add_layer_with_key(&mut self, key: u64, gfx: u64, name: String) -> Option<(u64, ObjAction)> {
        self.graphics.get_mut(&gfx)?.layers.push(key);
        let data = LayerData {
            gfx,
            name
        };
        self.layers.insert(key, Layer {
            data: data.clone(), 
        });
        Some((key, ObjAction::addition(key, data)))
    }

    pub fn delete_layer(&mut self, key: u64) -> Option<()> {
        let layer = self.layers.remove(&key)?;
        self.graphics.get_mut(&layer.data.gfx)?.layers.retain(|layer| *layer != key);
        None
    }
    
    pub fn set_layer_data(&mut self, key: u64, data: LayerData) -> Option<ObjAction> {
        let layer = self.layers.get_mut(&key)?;
        let res = ObjAction::modification(key, layer.data.clone(), data.clone());
        layer.data = data;
        Some(res)
    }

}
