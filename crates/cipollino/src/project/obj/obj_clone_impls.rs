
use std::sync::Arc;

use crate::{project::{saveload::{asset_file::AssetFile, load::LoadingMetadata}, Project}, util::bson::{bson_get, bson_to_u64, u64_to_bson}};
use super::{child_obj::{ChildObj, HasRootAsset}, DynObjPtr, Obj, ObjBox, ObjClone, ObjPtr, ObjSerialize, ToRawData};
use bson::bson;
use crate::project::obj::obj_list::ObjListTrait;

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

    fn obj_deserialize(project: &mut Project, data: &bson::Bson, parent: DynObjPtr, asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<Self> {
        let mut res = Vec::new(); 
        let elems = if let Some(elems) = data.as_array() {
            elems
        } else {
            metadata.deserialization_error("Array corrupted.", parent.key);
            return None;
        };
        for elem in elems { 
            res.push(T::obj_deserialize(project, elem, parent, asset_file, metadata)?);
        }
        Some(res)
    }

}

impl<T: ToRawData> ToRawData for Vec<T> {

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

    fn obj_deserialize(_project: &mut Project, data: &bson::Bson, parent: DynObjPtr, _asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<Self> {
        match bson::from_bson(data.clone()) {
            Ok(val) => val,
            Err(msg) => {
                metadata.deserialization_error(msg.to_string(), parent.key);
                None
            },
        }
    }

}

impl<T: PrimitiveObjClone> ToRawData for T {

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

impl<T: Obj + HasRootAsset> ObjSerialize for ObjPtr<T> {

    fn obj_serialize(&self, project: &Project, _asset_file: &mut AssetFile) -> bson::Bson {
        let root = T::get_root_asset(project, *self).unwrap();
        bson!({
            "key": u64_to_bson(self.key),
            "asset": u64_to_bson(root.key)
        })
    }
    
    fn obj_serialize_full(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson {
        self.obj_serialize(project, asset_file)
    }

    fn obj_deserialize(project: &mut Project, data: &bson::Bson, _parent: DynObjPtr, _asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<Self> {
        let key_data = bson_get(data, "key")?;
        let key = bson_to_u64(key_data)?;
        let asset_data = bson_get(data, "asset")?;
        let asset_key = bson_to_u64(asset_data)?;
        let asset_ptr = ObjPtr::from_key(asset_key); 
        if let Err(err) = <T::RootAsset as Obj>::ListType::load(project, asset_ptr, metadata) {
            metadata.error(err);
        }

        Some(ObjPtr::from_key(key))
    }

}

impl<T: Obj> ToRawData for ObjPtr<T> {

    type RawData = ObjPtr<T>;
    fn to_raw_data(&self, _project: &Project) -> Self::RawData {
        *self
    }

    fn from_raw_data(_project: &mut Project, data: &Self::RawData) -> Self {
        *data
    }

}

pub fn serialize_obj_box<T: Obj + ObjSerialize>(obj: &ObjBox<T>, project: &Project) -> bson::Bson {
    let ptr = T::get_list(project).use_obj_file_ptrs(|ptrs| *ptrs.get(&obj.ptr).expect("pointer missing. might be caused by parent being saved before child."));
    bson!({
        "key": u64_to_bson(obj.ptr.key),
        "ptr": u64_to_bson(ptr)
    })
}

pub fn serialize_obj_box_full<T: Obj + ObjSerialize>(obj: &ObjBox<T>, project: &Project, asset_file: &mut AssetFile) -> bson::Bson {
    let list = T::get_list(project);
    let new_page_ptr = asset_file.alloc_page().expect("could not allocate page.");
    list.use_obj_file_ptrs(|ptrs| ptrs.insert(obj.ptr, new_page_ptr));
    let obj_data = obj.get(project).obj_serialize_full(project, asset_file);
    let ptr = list.use_obj_file_ptrs(|ptrs| *ptrs.get(&obj.ptr).unwrap());
    let _ = asset_file.set_obj_data(ptr, obj_data.as_document().expect("object must serialize to a bson document").clone());
    serialize_obj_box(obj, project)
}

pub fn deserialize_obj_box<T: Obj + ObjSerialize>(project: &mut Project, data: &bson::Bson, parent: DynObjPtr, asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<(T, ObjPtr<T>)> {
    let make_error_msg = |msg| {
        format!("Could not deserialize {} ObjBox ({}).", T::type_name(), msg)
    };

    let page_ptr = if let Some(page_ptr) = bson_get(data, "ptr") {
        page_ptr
    } else {
        metadata.deserialization_error(make_error_msg("pointer missing"), parent.key);
        return None;
    };
    let page_ptr = if let Some(page_ptr) = bson_to_u64(page_ptr) {
        page_ptr
    } else {
        metadata.deserialization_error(make_error_msg("pointer must be u64."), parent.key);
        return None;
    };
    metadata.curr_ptr = page_ptr;

    let ptr = if let Some(key) = bson_get(data, "key") {
        let key = if let Some(key) = bson_to_u64(key) {
            key
        } else {
            metadata.deserialization_error("key must be u64.", parent.key); 
            return None;
        };
        let ptr = ObjPtr::from_key(key);
        if T::get_list(project).get(ptr).is_some() {
            T::get_list_mut(project).next_ptr()
        } else {
            *T::get_list_mut(project).curr_key() = (*T::get_list_mut(project).curr_key()).max(key + 1);
            ptr
        }
    } else {
        T::get_list_mut(project).next_ptr()
    }; 

    T::get_list_mut(project).use_obj_file_ptrs(|ptrs| ptrs.insert(ptr, page_ptr));

    let obj_data = match asset_file.get_obj_data(page_ptr) {
        Ok(obj_data) => obj_data,
        Err(msg) => {
            metadata.deserialization_error(msg, parent.key);
            return None;
        }
    };
    let obj = T::obj_deserialize(project, &bson::Bson::Document(obj_data), ptr.into(), asset_file, metadata)?;
    Some((obj, ptr))
}

impl<T: ChildObj + ObjSerialize> ObjSerialize for ObjBox<T> {

    fn obj_serialize(&self, project: &Project, _asset_file: &mut AssetFile) -> bson::Bson {
        serialize_obj_box(self, project)
    }

    fn obj_serialize_full(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson {
        serialize_obj_box_full(self, project, asset_file) 
    }

    fn obj_deserialize(project: &mut Project, data: &bson::Bson, parent: DynObjPtr, asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<Self> {
        let prev_metadata_curr_ptr = metadata.curr_ptr;
        let (mut obj, ptr) = if let Some((obj, ptr)) = deserialize_obj_box::<T>(project, data, parent, asset_file, metadata) {
            (obj, ptr)
        } else {
            metadata.curr_ptr = prev_metadata_curr_ptr;
            return None;
        };
        metadata.curr_ptr = prev_metadata_curr_ptr;
        *obj.parent_mut() = parent.into();
        Some(T::get_list_mut(project).add_with_ptr(obj, ptr)) // TODO: triggers an autosave
    }

}

impl<T: Obj + ToRawData> ToRawData for ObjBox<T> {

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
