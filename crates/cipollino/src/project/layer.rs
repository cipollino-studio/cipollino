
use project_macros::{ObjClone, ObjSerialize, Object};
use unique_type_id::UniqueTypeId;

use super::{action::ObjAction, frame::Frame, graphic::Graphic, obj::{child_obj::{ChildObj, HasRootAsset}, obj_clone_impls::PrimitiveObjClone, DynObjPtr, Obj, ObjClone, ObjList, ObjPtr, ObjSerialize}, sound_instance::SoundInstance, ObjBox, Project};

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub enum LayerKind {
    Animation,
    Audio,
    Group
}

impl PrimitiveObjClone for LayerKind {}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub enum LayerParent {
    Graphic(ObjPtr<Graphic>),
    Layer(ObjPtr<Layer>)
}

impl PrimitiveObjClone for LayerParent {}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub enum BlendingMode {
    // Normal
    Normal,

    // Lighten
    Add,
    ColorDodge,

    // Darken
    Multiply,
    ColorBurn,

    // Contrast
    Overlay
}

impl BlendingMode {

    pub fn name(&self) -> &'static str {
        match self {
            BlendingMode::Normal => "Normal",
            BlendingMode::Add => "Add",
            BlendingMode::ColorDodge => "Color Dodge",
            BlendingMode::Multiply => "Multiply",
            BlendingMode::ColorBurn => "Color Burn",
            BlendingMode::Overlay => "Overlay",
        } 
    }

}

impl PrimitiveObjClone for BlendingMode {}

#[derive(Object, Clone, ObjClone, ObjSerialize, UniqueTypeId)]
pub struct Layer {
    #[parent]
    pub parent: LayerParent,
    #[field]
    pub name: String,
    #[field]
    pub show: bool,
    #[field]
    pub open: bool, // Only used for layer groups
    #[field]
    pub kind: LayerKind,

    #[field]
    pub alpha: f32,
    #[field]
    pub blending: BlendingMode,

    pub frames: Vec<ObjBox<Frame>>,
    pub sound_instances: Vec<ObjBox<SoundInstance>>,
    pub layers: Vec<ObjBox<Layer>>
}

impl Layer {

    pub fn get_frame_at(&self, project: &Project, time: i32) -> Option<&ObjBox<Frame>> {
        let mut best_frame = None;
        let mut best_time = -1;
        for frame in &self.frames {
            if frame.get(project).time <= time && frame.get(project).time > best_time {
                best_frame = Some(frame);
                best_time = frame.get(project).time;
            }
        }
        best_frame
    }

    pub fn get_frame_exactly_at(&self, project: &Project, time: i32) -> Option<&ObjBox<Frame>> {
        for frame in &self.frames {
            if frame.get(project).time == time {
                return Some(frame);
            }
        }
        None
    }

    pub fn get_frame_before(&self, project: &Project, time: i32) -> Option<&ObjBox<Frame>> {
        let mut best_frame = None;
        let mut best_time = -1;
        for frame in &self.frames {
            if frame.get(project).time < time && frame.get(project).time > best_time {
                best_frame = Some(frame);
                best_time = frame.get(project).time;
            }
        }
        best_frame
    }
    
    pub fn get_frame_after(&self, project: &Project, time: i32) -> Option<&ObjBox<Frame>> {
        let mut best_frame = None;
        let mut best_time = i32::MAX;
        for frame in &self.frames {
            if frame.get(project).time > time && frame.get(project).time < best_time {
                best_frame = Some(frame);
                best_time = frame.get(project).time;
            }
        }
        best_frame
    }

    fn inside(project: &Project, layer: ObjPtr<Layer>, parent: LayerParent) -> bool {
        match parent {
            LayerParent::Graphic(_) => false,
            LayerParent::Layer(parent_layer) => {
                if parent_layer == layer {
                    return true;
                }
                if let Some(parent_layer) = project.layers.get(parent_layer) {
                    Self::inside(project, layer, parent_layer.parent)
                } else {
                    false
                }
            },
        }
    }

    pub fn transfer(project: &mut Project, layer: ObjPtr<Self>, new_parent: LayerParent) -> Option<ObjAction> {
        if let LayerParent::Layer(new_layer) = new_parent {
            if Self::inside(project, new_layer, LayerParent::Layer(layer)) {
                return None;
            }
        }
        <Self as ChildObj>::transfer(project, layer, new_parent)
    }

    pub fn requires_offscreen_render(&self) -> bool {
        self.alpha < 0.999 || self.blending != BlendingMode::Normal || self.kind == LayerKind::Group
    }

}

impl From<DynObjPtr> for LayerParent {
    
    fn from(value: DynObjPtr) -> Self {
        if value.is::<Layer>() {
            LayerParent::Layer(value.into())
        } else if value.is::<Graphic>() {
            LayerParent::Graphic(value.into())
        } else {
            panic!("invalid ptr cast.");
        }
    }

}

impl ChildObj for Layer {
    type Parent = LayerParent;

    fn parent(&self) -> Self::Parent {
        self.parent
    }

    fn parent_mut(&mut self) -> &mut Self::Parent {
        &mut self.parent
    }

    fn get_list_in_parent(project: &Project, parent: Self::Parent) -> Option<&Vec<ObjBox<Self>>> {
        match parent {
            LayerParent::Graphic(gfx) => Some(&project.graphics.get(gfx)?.layers),
            LayerParent::Layer(layer) => Some(&project.layers.get(layer)?.layers),
        }
    }

    fn get_list_in_parent_mut(project: &mut Project, parent: Self::Parent) -> Option<&mut Vec<ObjBox<Self>>> {
        match parent {
            LayerParent::Graphic(gfx) => Some(&mut project.graphics.get_mut(gfx)?.layers),
            LayerParent::Layer(layer) => Some(&mut project.layers.get_mut(layer)?.layers),
        }
    }

}

impl Default for Layer {

    fn default() -> Self {
        Self {
            parent: LayerParent::Graphic(ObjPtr::null()),
            name: "Layer".to_owned(),
            show: true,
            open: true,
            kind: LayerKind::Animation,
            alpha: 1.0,
            blending: BlendingMode::Normal,
            frames: Vec::new(),
            sound_instances: Vec::new(),
            layers: Vec::new()
        }
    }

}

impl HasRootAsset for Layer {
    type RootAsset = Graphic;

    fn get_root_asset(project: &Project, ptr: ObjPtr<Self>) -> Option<ObjPtr<Self::RootAsset>> {
        let layer = project.layers.get(ptr)?;
        match layer.parent {
            LayerParent::Graphic(gfx) => Some(gfx),
            LayerParent::Layer(layer) => Layer::get_root_asset(project, layer),
        }
    }
}
