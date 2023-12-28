
use super::{Project, action::ObjAction, ObjData};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Layer {
    pub data: LayerData,
    pub frames: Vec<u64>
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
            frames: Vec::new()
        });
        Some((key, ObjAction::addition(key, data)))
    }

    pub fn delete_layer(&mut self, key: u64) -> Option<Vec<ObjAction>> {
        let layer = self.layers.remove(&key)?;
        let mut acts = Vec::new();
        for frame in layer.frames {
            if let Some(mut frame_acts) = self.delete_frame(frame) {
                acts.append(&mut frame_acts);
            }
        }
        if let Some(gfx) = self.graphics.get_mut(&layer.data.gfx) {
            gfx.layers.retain(|layer| *layer != key);
        }
        acts.push(ObjAction::deletion(key, layer.data));
        Some(acts) 
    }
    
    pub fn set_layer_data(&mut self, key: u64, data: LayerData) -> Option<ObjAction> {
        let layer = self.layers.get_mut(&key)?;
        let res = ObjAction::modification(key, layer.data.clone(), data.clone());
        layer.data = data;
        Some(res)
    }

    pub fn get_frame_at(&self, layer: u64, time: i32) -> Option<u64> {
        let layer = self.layers.get(&layer)?;
        let mut best_frame = 0;
        let mut best_time = -1;
        for frame_key in &layer.frames {
            let frame = self.frames.get(frame_key)?;
            if frame.data.time <= time && frame.data.time > best_time {
                best_frame = *frame_key;
                best_time = frame.data.time;
            }
        }
        if best_frame == 0 {
            None
        } else {
            Some(best_frame)
        }
    }

    pub fn get_frame_exactly_at(&self, layer: u64, time: i32) -> Option<u64> {
        let layer = self.layers.get(&layer)?;
        for frame_key in &layer.frames {
            let frame = self.frames.get(frame_key)?;
            if frame.data.time == time {
                return Some(*frame_key);
            }
        }
        None
    }

    pub fn get_frame_before(&self, layer: u64, time: i32) -> Option<u64> {
        let layer = self.layers.get(&layer)?;
        let mut best_frame = 0;
        let mut best_time = -1;
        for frame_key in &layer.frames {
            let frame = self.frames.get(frame_key)?;
            if frame.data.time < time && frame.data.time > best_time {
                best_frame = *frame_key;
                best_time = frame.data.time;
            }
        }
        if best_frame == 0 {
            None
        } else {
            Some(best_frame)
        } 
    }
    
    pub fn get_frame_after(&self, layer: u64, time: i32) -> Option<u64> {
        let layer = self.layers.get(&layer)?;
        let mut best_frame = 0;
        let mut best_time = i32::MAX;
        for frame_key in &layer.frames {
            let frame = self.frames.get(frame_key)?;
            if frame.data.time > time && frame.data.time < best_time {
                best_frame = *frame_key;
                best_time = frame.data.time;
            }
        }
        if best_frame == 0 {
            None
        } else {
            Some(best_frame)
        } 
    }

}
