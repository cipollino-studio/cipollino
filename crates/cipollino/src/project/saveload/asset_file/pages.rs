
use std::io;

use super::{AssetFile, FIRST_FREE_PAGE};

pub const FILE_PAGE_DATA_SIZE: u64 = 128;
pub const FILE_PAGE_DATA_SIZE_USIZE: usize = FILE_PAGE_DATA_SIZE as usize;
pub const FILE_PAGE_METADATA_SIZE: u64 = 8;

pub const FILE_PAGE_NEXT_PTR_OFFSET: u64 = 0;
pub const FILE_PAGE_DATA_OFFSET: u64 = FILE_PAGE_METADATA_SIZE;

impl AssetFile {

    pub fn alloc_page(&mut self) -> Result<u64, io::Error> {
        let first_free_page = self.read_u64_from(FIRST_FREE_PAGE)?;
        if first_free_page != 0 {
            let next_free_page = self.read_u64_from(first_free_page + FILE_PAGE_NEXT_PTR_OFFSET)?;
            self.write_u64_to(FIRST_FREE_PAGE, next_free_page)?;

            self.write_u64_to(first_free_page + FILE_PAGE_NEXT_PTR_OFFSET, 0)?;
            
            return Ok(first_free_page);
        }
        let new_page = self.file_size()?;
        self.cursor_to(new_page)?;
        self.write_u64(0)?; // Next page pointer
        self.write(&vec![0; FILE_PAGE_DATA_SIZE as usize])?; // Blank data

        Ok(new_page)
    }

    pub fn free_page(&mut self, page: u64) -> Result<(), io::Error> {
        let first_free_page = self.read_u64_from(FIRST_FREE_PAGE)?;
        self.write_u64_to(page + FILE_PAGE_NEXT_PTR_OFFSET, first_free_page)?;
        self.write_u64_to(FIRST_FREE_PAGE, page)?;
        self.cursor_to(page + FILE_PAGE_DATA_OFFSET)?;
        self.write(&vec![0xFF; FILE_PAGE_DATA_SIZE as usize])?; // Fill page with nonsense for debugging & privacy purposes 
        Ok(())
    }

    pub fn free_page_chain(&mut self, page: u64) -> Result<(), io::Error> {
        let mut curr_page = page;
        while curr_page != 0 {
            let next_page = self.read_u64_from(curr_page + FILE_PAGE_NEXT_PTR_OFFSET)?;
            self.free_page(curr_page)?;
            curr_page = next_page;
        }
        Ok(())
    }

}
