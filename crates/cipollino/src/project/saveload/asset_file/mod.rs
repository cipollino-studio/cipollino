
use std::{fs::File, path::Path};

use self::pages::{FILE_PAGE_DATA_OFFSET, FILE_PAGE_DATA_SIZE_USIZE, FILE_PAGE_NEXT_PTR_OFFSET};

pub mod io;
pub mod pages;

const MAGIC_BYTES_PTR: u64 = 0;
const ASSET_TYPE_MAGIC_BYTES_PTR: u64 = 4;
const VERSION_PTR: u64 = 8;
const ROOT_OBJ_PTR: u64 = 16;
const ROOT_OBJ_KEY: u64 = 24;
const FIRST_FREE_PAGE: u64 = 32;

const MAGIC_BYTES: [u8; 4] = *b"cipp";
const LATEST_VERSION: u64 = 1;

pub struct AssetFile {
    file: File,
    pub root_obj_ptr: u64,
    pub root_obj_key: u64,
    pub version: u64
}

impl AssetFile {

    pub fn create<P: AsRef<Path>>(path: P, key: u64, asset_type_magic_bytes: &[u8; 4]) -> Result<Self, String> {
        let file = File::options().read(true).write(true).create(true).open(path).map_err(|err| err.to_string())?;
        let mut res = AssetFile {
            file,
            root_obj_ptr: 0,
            root_obj_key: key,
            version: LATEST_VERSION
        };
        res.cursor_to(0)?;
        res.write(&MAGIC_BYTES)?; // Magic bytes
        res.write(asset_type_magic_bytes)?; // Asset type magic bytes
        res.write_u64(LATEST_VERSION)?; // Version
        res.write_u64(0)?; // Root object pointer
        res.write_u64(key)?; // Root object key
        res.write_u64(0)?; // First free block

        let root_obj_page = res.alloc_page()?;
        res.write_u64_to(ROOT_OBJ_PTR, root_obj_page)?;
        res.root_obj_ptr = root_obj_page;
        Ok(res)
    }

    pub fn open<P: AsRef<Path>>(path: P, asset_type_magic_bytes: &[u8; 4], asset_type_name: &'static str) -> Result<Self, String> {
        let file = File::options().read(true).write(true).open(path).map_err(|err| err.to_string())?;
        let mut res = AssetFile {
            file,
            root_obj_ptr: 0,
            root_obj_key: 0,
            version: 0
        };

        res.cursor_to(MAGIC_BYTES_PTR)?;
        let magic = res.read::<4>()?;
        if magic != MAGIC_BYTES {
            return Err("Not a cipollino asset file.".to_owned());
        }

        res.cursor_to(ASSET_TYPE_MAGIC_BYTES_PTR)?;
        let asset_type_magic = res.read::<4>()?; 
        if &asset_type_magic != asset_type_magic_bytes {
            return Err(format!("Not a cipollino {} file.", asset_type_name));
        }

        res.version = res.read_u64_from(VERSION_PTR)?;
        res.root_obj_ptr = res.read_u64_from(ROOT_OBJ_PTR)?;
        res.root_obj_key = res.read_u64_from(ROOT_OBJ_KEY)?;
        Ok(res)
    }

    pub fn get_obj_data(&mut self, first_page_ptr: u64) -> Result<bson::Document, String> {
        let mut bytes = Vec::new();
        let mut curr_page = first_page_ptr;
        loop {
            self.cursor_to(curr_page + FILE_PAGE_DATA_OFFSET)?;
            let page_data = self.read::<FILE_PAGE_DATA_SIZE_USIZE>()?;
            bytes.extend_from_slice(&page_data);
            curr_page = self.read_u64_from(curr_page)?;
            if curr_page == 0 {
                break;
            }
        }
        bson::Document::from_reader(bytes.as_slice()).map_err(|err| err.to_string())
    } 

    pub fn set_obj_data(&mut self, first_page_ptr: u64, data: bson::Document) -> Result<(), String> {
        let mut data_bytes = Vec::new(); 
        data.to_writer(&mut data_bytes).expect("serialization should not fail");
        let mut data_bytes = data_bytes.as_slice();
        let mut curr_page = first_page_ptr;
        loop {
            let (curr_page_data, rest) = data_bytes.split_at(FILE_PAGE_DATA_SIZE_USIZE.min(data_bytes.len()));
            self.cursor_to(curr_page + FILE_PAGE_DATA_OFFSET)?;
            self.write(curr_page_data)?;

            data_bytes = rest; 
            if data_bytes.len() == 0 {
                break;
            }
            let next_page = self.read_u64_from(curr_page + FILE_PAGE_NEXT_PTR_OFFSET)?;
            if next_page == 0 {
                let new_page = self.alloc_page()?;
                self.write_u64_to(curr_page + FILE_PAGE_NEXT_PTR_OFFSET, new_page)?;
                curr_page = new_page;
            } else {
                curr_page = next_page;
            }
        }

        let remaining_pages = self.read_u64_from(curr_page + FILE_PAGE_NEXT_PTR_OFFSET)?;
        if remaining_pages != 0 {
            self.free_page_chain(remaining_pages)?;
            self.write_u64_to(curr_page + FILE_PAGE_NEXT_PTR_OFFSET, 0)?;
        }

        Ok(())
    }

    pub fn delete_obj(&mut self, first_page_ptr: u64) -> Result<(), String> {
        self.free_page_chain(first_page_ptr)
    }

    pub fn set_root_obj_key(&mut self, key: u64) -> Result<(), String> {
        self.root_obj_key = key;
        self.write_u64_to(ROOT_OBJ_KEY, key)
    }

}
