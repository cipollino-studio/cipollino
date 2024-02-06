
use std::marker::PhantomData;

use crate::project::Project;
use super::{Obj, ObjBox, ObjClone, ObjPtr};
use serde_json::json;

impl<T: ObjClone> ObjClone for Vec<T> {

    fn obj_serialize(&self, project: &Project) -> serde_json::Value {
        self.iter().map(|elem| elem.obj_serialize(project)).collect()
    }

    fn obj_deserialize(project: &mut Project, data: &serde_json::Value) -> Option<Self> {
        let mut res = Vec::new(); 
        for elem in data.as_array()? {
            res.push(T::obj_deserialize(project, elem)?);
        }
        Some(res)
    }
    
}

trait Simple : Clone + serde::Serialize + for<'a> serde::Deserialize<'a> {}

impl<T: Simple> ObjClone for T {

    fn obj_serialize(&self, _project: &Project) -> serde_json::Value {
        json! {self}
    }

    fn obj_deserialize(_project: &mut Project, data: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(data.clone()).ok()
    }

}

impl Simple for bool {}
impl Simple for u32 {}
impl Simple for u64 {}
impl Simple for i32 {}
impl Simple for f32 {}
impl Simple for String {}
impl Simple for glam::Vec2 {}
impl Simple for glam::Vec3 {}
impl Simple for glam::Vec4 {}

impl<T: Obj> ObjClone for ObjPtr<T> {

    fn obj_serialize(&self, _project: &Project) -> serde_json::Value {
        json!{self.key}
    }

    fn obj_deserialize(_project: &mut Project, data: &serde_json::Value) -> Option<Self> {
        Some(Self {
            key: data.as_u64()?,
            _marker: PhantomData
        })
    }

}

impl<T: Obj> ObjClone for ObjBox<T> {

    fn obj_clone(&self, project: &mut Project) -> Self {
        let list = T::get_list(project);
        let obj = list.get(self.ptr).unwrap();
        let obj_clone = obj.clone(); // Hack to get around the borrow checker
        let obj_clone = obj_clone.obj_clone(project);
        T::get_list_mut(project).add(obj_clone)
    }

    fn obj_serialize(&self, project: &Project) -> serde_json::Value {
        json! {{
            "ptr": self.ptr.obj_serialize(project),
            "obj": self.get(project).obj_serialize(project)
        }}
    }

    fn obj_deserialize(project: &mut Project, data: &serde_json::Value) -> Option<Self> {
        let ptr = ObjPtr::<T>::obj_deserialize(project, data.get("ptr")?)?;
        let obj = T::obj_deserialize(project, data.get("obj")?)?;
        Some(T::get_list_mut(project).add_with_ptr(obj, ptr))
    }

}
