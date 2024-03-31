
use std::{fs::File, path::Path};

use self::pages::{FILE_PAGE_DATA_OFFSET, FILE_PAGE_DATA_SIZE_USIZE, FILE_PAGE_NEXT_PTR_OFFSET};

pub mod io;
pub mod pages;

const ROOT_OBJ_PTR: u64 = 0;
const ROOT_OBJ_KEY: u64 = 8;
const FIRST_FREE_PAGE: u64 = 16;

pub struct AssetFile {
    file: File,
    pub root_obj_ptr: u64,
    pub root_obj_key: u64
}

impl AssetFile {

    pub fn create<P: AsRef<Path>>(path: P, key: u64) -> Result<Self, std::io::Error> {
        let file = File::options().read(true).write(true).create(true).open(path)?;
        let mut res = AssetFile {
            file,
            root_obj_ptr: 0,
            root_obj_key: key
        };
        res.cursor_to(0)?;
        res.write_u64(0)?; // Root object
        res.write_u64(key)?; // Root object key
        res.write_u64(0)?; // First free block

        let root_obj_page = res.alloc_page()?;
        res.write_u64_to(ROOT_OBJ_PTR, root_obj_page)?;
        res.root_obj_ptr = root_obj_page;
        Ok(res)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::options().read(true).write(true).open(path)?;
        let mut res = AssetFile {
            file,
            root_obj_ptr: 0,
            root_obj_key: 0
        };
        res.root_obj_ptr = res.read_u64_from(ROOT_OBJ_PTR)?;
        res.root_obj_key = res.read_u64_from(ROOT_OBJ_KEY)?;
        Ok(res)
    }

    pub fn get_obj_data(&mut self, first_page_ptr: u64) -> Result<Option<bson::Document>, std::io::Error> {
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
        Ok(bson::Document::from_reader(bytes.as_slice()).ok())
    } 

    pub fn set_obj_data(&mut self, first_page_ptr: u64, data: bson::Document) -> Result<(), std::io::Error> {
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

    pub fn delete_obj(&mut self, first_page_ptr: u64) -> Result<(), std::io::Error> {
        self.free_page_chain(first_page_ptr)
    }

}
