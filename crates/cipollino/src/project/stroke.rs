
use glam::{Vec2, Mat4, vec3, vec2};
use project_macros::{ObjClone, ObjSerialize, Object};

use super::{action::ObjAction, frame::Frame, obj::{child_obj::ChildObj, Obj, ObjBox, ObjClone, ObjList, ObjPtr, ObjPtrAny, ObjSerialize}, palette::PaletteColor, Project};

#[derive(Clone, Copy, ObjClone, Default, ObjSerialize)]
pub struct StrokePoint {
    pub a: Vec2,
    pub pt: Vec2,
    pub b: Vec2
}

#[derive(Clone, Copy, Debug)]
pub enum StrokeColor {
    Color(glam::Vec4),
    Palette(ObjPtr<PaletteColor>, glam::Vec4) 
}

impl StrokeColor {

    pub fn get_color(&self, project: &Project) -> glam::Vec4 {
        match self {
            Self::Color(color) => *color,
            Self::Palette(ptr, backup_color) => {
                let color = project.palette_colors.get(*ptr).map(|color| color.color);
                color.unwrap_or(*backup_color)
            }
        }
    }
    
}

impl ObjClone for StrokeColor {}

impl ObjSerialize for StrokeColor {

    fn obj_serialize(&self, project: &Project) -> bson::Bson {
        match self {
            Self::Color(color) => bson::bson!([color.x, color.y, color.z, color.w]),
            Self::Palette(ptr, backup_color) => bson::bson!({
                "color": ptr.obj_serialize(project),
                "backup": [backup_color.x, backup_color.y, backup_color.z, backup_color.w]
            })
        }
    }

    fn obj_deserialize(project: &mut Project, data: &bson::Bson, parent: ObjPtrAny) -> Option<Self> {
        if let Some(arr) = data.as_array() {
            let mut color = [0.0; 4];
            color[3] = 1.0;
            for i in 0..(arr.len().min(4)) {
                color[i] = arr[i].as_f64()? as f32;
            } 
            let color = glam::Vec4::from_slice(&color);
            Some(StrokeColor::Color(color))
        } else if let Some(obj) = data.as_document() {
            let ptr = obj.get(&"color".to_owned()).map(|data| ObjPtr::<PaletteColor>::obj_deserialize(project, data, parent).unwrap_or(ObjPtr::null())).unwrap_or(ObjPtr::null());
            let backup = obj.get(&"backup".to_owned()).map(|data| glam::Vec4::obj_deserialize(project, data, parent).unwrap_or(glam::vec4(0.0, 0.0, 0.0, 1.0))).unwrap_or(glam::vec4(0.0, 0.0, 0.0, 1.0));
            Some(Self::Palette(ptr, backup))
        } else {
            None
        }
    }
}

#[derive(Object, Clone, ObjClone, ObjSerialize)]
pub struct Stroke {
    #[parent]
    pub frame: ObjPtr<Frame>,
    #[field]
    pub color: StrokeColor,
    #[field]
    pub r: f32,
    #[field]
    pub filled: bool,
    pub points: Vec<Vec<StrokePoint>>,
}

impl Stroke {

    pub fn iter_point_pairs(&self) -> impl Iterator<Item = (StrokePoint, StrokePoint)> + '_ {
        self.points.iter().flat_map(|arr| arr.windows(2).map(|arr| (arr[0], arr[1])))
    }

    pub fn transform(project: &mut Project, stroke_ptr: ObjPtr<Stroke>, trans: Mat4) -> Option<ObjAction> {
        let transform_vec2 = |pt: Vec2, mat: Mat4| {
            let v3 = mat.transform_point3(vec3(pt.x, pt.y, 0.0));
            vec2(v3.x, v3.y)
        };

        project.remeshes_needed.insert(stroke_ptr);
        project.strokes.get_then_mut(stroke_ptr, move |stroke| {
            for chain in &mut stroke.points {
                for pt in chain {
                    pt.a = transform_vec2(pt.a, trans);
                    pt.pt = transform_vec2(pt.pt, trans);
                    pt.b = transform_vec2(pt.b, trans);
                }
            }
            ObjAction::new(move |proj| {
                Stroke::transform(proj, stroke_ptr, trans);
            }, move |proj| {
                Stroke::transform(proj, stroke_ptr, trans.inverse());
            })
        })
    }

}

impl ChildObj for Stroke {
    type Parent = Frame;

    fn parent_mut(&mut self) -> &mut ObjPtr<Self::Parent> {
        &mut self.frame
    }

    fn get_list_in_parent(parent: &Self::Parent) -> &Vec<ObjBox<Self>> {
        &parent.strokes
    }

    fn get_list_in_parent_mut(parent: &mut Self::Parent) -> &mut Vec<ObjBox<Self>> {
        &mut parent.strokes
    }

}

impl Default for Stroke {

    fn default() -> Self {
        Self {
            frame: ObjPtr::null(),
            color: StrokeColor::Color(glam::vec4(0.0, 0.0, 0.0, 1.0)),
            r: 0.05,
            filled: false,
            points: Vec::new()
        }
    }

}
