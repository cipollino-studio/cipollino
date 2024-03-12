
use std::sync::Arc;

use crate::project::Project;
use super::{child_obj::ChildObj, Obj, ObjBox, ObjClone, ObjPtr, ObjPtrAny, ObjSerialize};
use serde_json::json;

impl<T: ObjClone> ObjClone for Vec<T> {}

impl<T: ObjSerialize> ObjSerialize for Vec<T> {

    fn obj_serialize(&self, project: &Project) -> serde_json::Value {
        self.iter().map(|elem| elem.obj_serialize(project)).collect()
    }

    fn obj_deserialize(project: &mut Project, data: &serde_json::Value, parent: ObjPtrAny) -> Option<Self> {
        let mut res = Vec::new(); 
        for elem in data.as_array()? {
            res.push(T::obj_deserialize(project, elem, parent)?);
        }
        Some(res)
    }
    
}

pub trait PrimitiveObjClone : Clone + serde::Serialize + for<'a> serde::Deserialize<'a> {}

impl<T: PrimitiveObjClone> ObjClone for T {}

impl<T: PrimitiveObjClone> ObjSerialize for T {

    fn obj_serialize(&self, _project: &Project) -> serde_json::Value {
        json! {self}
    }

    fn obj_deserialize(_project: &mut Project, data: &serde_json::Value, _parent: ObjPtrAny) -> Option<Self> {
        serde_json::from_value(data.clone()).ok()
    }

}
impl PrimitiveObjClone for bool {}
impl PrimitiveObjClone for u32 {}
impl PrimitiveObjClone for u64 {}
impl PrimitiveObjClone for i32 {}
impl PrimitiveObjClone for i64 {}
impl PrimitiveObjClone for f32 {}
impl PrimitiveObjClone for String {}
impl PrimitiveObjClone for glam::Vec2 {}
impl PrimitiveObjClone for glam::Vec3 {}
impl PrimitiveObjClone for glam::Vec4 {}

impl<T: Obj> ObjClone for ObjPtr<T> {}

impl<T: Obj> ObjClone for ObjBox<T> {

    fn obj_clone(&self, project: &mut Project) -> Self {
        let list = T::get_list(project);
        let obj = list.get(self.ptr).unwrap();
        let obj_clone = obj.clone(); // Hack to get around the borrow checker
        let obj_clone = obj_clone.obj_clone(project);
        T::get_list_mut(project).add(obj_clone)
    }

}

impl<T: Obj> ObjSerialize for ObjPtr<T> {

    fn obj_serialize(&self, _project: &Project) -> serde_json::Value {
        json!(self.key)
    }

    fn obj_deserialize(_project: &mut Project, data: &serde_json::Value, _parent: ObjPtrAny) -> Option<Self> {
        Some(Self::from_key(data.as_u64()?))
    }
}

impl<T: ChildObj + ObjSerialize> ObjSerialize for ObjBox<T> {

    fn obj_serialize(&self, project: &Project) -> serde_json::Value {
        let mut data = self.get(project).obj_serialize(project);
        if let Some(map) = data.as_object_mut() {
            map.insert("key".to_owned(), json!(self.ptr.key));
        }
        data
    }

    fn obj_deserialize(project: &mut Project, data: &serde_json::Value, parent: ObjPtrAny) -> Option<Self> {
        
        let ptr = if let Some(key) = data.get("key") {
            let key = key.as_u64()?;
            let ptr = ObjPtr::from_key(key);
            if T::get_list(project).get(ptr).is_some() {
                T::get_list_mut(project).next_ptr()
            } else {
                T::get_list_mut(project).curr_key = T::get_list_mut(project).curr_key.max(key + 1);
                ptr
            }
        } else {
            T::get_list_mut(project).next_ptr()
        }; 
        
        let mut obj = T::obj_deserialize(project, data, ptr.into())?;
        *obj.parent_mut() = parent.into();
        Some(T::get_list_mut(project).add_with_ptr(obj, ptr))
    }

}

impl<T: serde::Serialize + for<'a> serde::Deserialize<'a>> PrimitiveObjClone for Arc<T> {}
