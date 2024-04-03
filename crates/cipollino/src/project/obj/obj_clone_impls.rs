
use std::sync::Arc;

use crate::{project::{saveload::{asset_file::AssetFile, load::LoadingMetadata}, Project}, util::bson::{bson_get, bson_to_u64, u64_to_bson}};
use super::{child_obj::ChildObj, Obj, ObjBox, ObjClone, ObjPtr, ObjPtrAny, ObjSerialize};
use bson::bson;

impl<T: ObjClone> ObjClone for Vec<T> {

    fn obj_clone(&self, project: &mut Project) -> Self {
        let mut res = Vec::new();
        for elem in self {
            res.push(elem.obj_clone(project));
        } 
        res
    }

}

impl<T: ObjSerialize> ObjSerialize for Vec<T> {

    fn obj_serialize(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson {
        self.iter().map(|elem| elem.obj_serialize(project, asset_file)).collect()
    }

    fn obj_serialize_full(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson {
        self.iter().map(|elem| elem.obj_serialize_full(project, asset_file)).collect()
    }

    fn obj_deserialize(project: &mut Project, data: &bson::Bson, parent: ObjPtrAny, asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<Self> {
        let mut res = Vec::new(); 
        for elem in data.as_array()? {
            res.push(T::obj_deserialize(project, elem, parent, asset_file, metadata)?);
        }
        Some(res)
    }

    type RawData = Vec<T::RawData>;
    
    fn to_raw_data(&self, project: &Project) -> Self::RawData {
        self.iter().map(|elem| elem.to_raw_data(project)).collect()
    }
    
    fn from_raw_data(project: &mut Project, data: &Self::RawData) -> Self {
        data.iter().map(|elem| T::from_raw_data(project, elem)).collect()
    }

    
}

pub trait PrimitiveObjClone : Clone + serde::Serialize + for<'a> serde::Deserialize<'a> + Send + Sync {}

impl<T: PrimitiveObjClone> ObjClone for T {}

impl<T: PrimitiveObjClone> ObjSerialize for T {

    fn obj_serialize(&self, _project: &Project, _asset_file: &mut AssetFile) -> bson::Bson {
        bson::to_bson(self).expect("serialization of primitive should not fail")
    }

    fn obj_serialize_full(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson {
        self.obj_serialize(project, asset_file)
    }

    fn obj_deserialize(_project: &mut Project, data: &bson::Bson, _parent: ObjPtrAny, _asset_file: &mut AssetFile, _metadata: &mut LoadingMetadata) -> Option<Self> {
        bson::from_bson(data.clone()).ok()
    }

    type RawData = T;

    fn to_raw_data(&self, _project: &Project) -> Self::RawData {
        self.clone()
    }

    fn from_raw_data(_project: &mut Project, data: &Self::RawData) -> Self {
        data.clone() 
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

    fn obj_serialize(&self, _project: &Project, _asset_file: &mut AssetFile) -> bson::Bson {
        u64_to_bson(self.key) 
    }
    
    fn obj_serialize_full(&self, _project: &Project, _asset_file: &mut AssetFile) -> bson::Bson {
        u64_to_bson(self.key) 
    }

    fn obj_deserialize(_project: &mut Project, data: &bson::Bson, _parent: ObjPtrAny, _asset_file: &mut AssetFile, _metadata: &mut LoadingMetadata) -> Option<Self> {
        Some(Self::from_key(bson_to_u64(data)?))
    }

    type RawData = ObjPtr<T>;
    fn to_raw_data(&self, _project: &Project) -> Self::RawData {
        *self
    }

    fn from_raw_data(_project: &mut Project, data: &Self::RawData) -> Self {
        *data
    }

}

impl<T: ChildObj + ObjSerialize> ObjSerialize for ObjBox<T> {

    fn obj_serialize(&self, project: &Project, _asset_file: &mut AssetFile) -> bson::Bson {
        let ptr = *T::get_list(project).obj_file_ptrs.borrow().get(&self.ptr).expect("pointer missing. might be caused by parent being saved before child.");
        bson!({
            "key": u64_to_bson(self.ptr.key),
            "ptr": u64_to_bson(ptr)
        })
    }

    fn obj_serialize_full(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson {
        let list = T::get_list(project);
        list.obj_file_ptrs.borrow_mut().insert(self.ptr, asset_file.alloc_page().expect("could not allocate page."));
        let obj_data = self.get(project).obj_serialize_full(project, asset_file);
        let ptr = *list.obj_file_ptrs.borrow().get(&self.ptr).unwrap();
        let _ = asset_file.set_obj_data(ptr, obj_data.as_document().expect("object must serialize to a bson document").clone());
        self.obj_serialize(project, asset_file)
    }

    fn obj_deserialize(project: &mut Project, data: &bson::Bson, parent: ObjPtrAny, asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<Self> {
        
        let page_ptr = bson_to_u64(bson_get(data, "ptr")?)?;

        let ptr = if let Some(key) = bson_get(data, "key") {
            let key = bson_to_u64(key)?; 
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

        T::get_list_mut(project).obj_file_ptrs.borrow_mut().insert(ptr, page_ptr);

        let obj_data = asset_file.get_obj_data(page_ptr).ok()??;
        let mut obj = T::obj_deserialize(project, &bson::Bson::Document(obj_data), ptr.into(), asset_file, metadata)?;
        *obj.parent_mut() = parent.into();
        Some(T::get_list_mut(project).add_with_ptr(obj, ptr)) // TODO: triggers an autosave
    }

    type RawData = (u64, T::RawData);
    fn to_raw_data(&self, project: &Project) -> Self::RawData {
        (self.ptr.key, self.get(project).to_raw_data(project))
    }

    fn from_raw_data(project: &mut Project, data: &Self::RawData) -> Self {
        let (key, obj_data) = data; 
        let obj = T::from_raw_data(project, obj_data);
        T::get_list_mut(project).add_with_ptr(obj, ObjPtr::from_key(*key))
    }

}

impl<T: serde::Serialize + for<'a> serde::Deserialize<'a> + Send + Sync> PrimitiveObjClone for Arc<T> {}
