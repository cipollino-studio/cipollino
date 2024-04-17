
use std::io::{Read, Seek, SeekFrom, Write};

use crate::project::saveload::asset_file::AssetFile;

impl AssetFile {

    pub fn cursor_to(&mut self, ptr: u64) -> Result<(), String> {
        self.file.seek(SeekFrom::Start(ptr)).map(|_| ()).map_err(|err| err.to_string())
    }
    
    pub fn cursor_ptr(&mut self) -> Result<u64, String> {
        self.file.seek(SeekFrom::Current(0)).map_err(|err| err.to_string())
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), String> {
        self.file.write(data).map_err(|err| err.to_string())?;
        Ok(())
    }
    
    pub fn write_u32(&mut self, val: u32) -> Result<(), String> {
        self.write(&val.to_le_bytes())?;
        Ok(())
    }

    pub fn write_u64(&mut self, val: u64) -> Result<(), String> {
        self.write(&val.to_le_bytes())?;
        Ok(())
    }

    pub fn write_u64_to(&mut self, ptr: u64, val: u64) -> Result<(), String> {
        self.cursor_to(ptr)?;
        self.write_u64(val)?;
        Ok(())
    }

    pub fn file_size(&mut self) -> Result<u64, String> {
        self.file.seek(SeekFrom::End(0)).map_err(|err| err.to_string())
    }

    pub fn read<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let mut res = [0; N];
        self.file.read(&mut res).map_err(|err| err.to_string())?;
        Ok(res)
    }

    pub fn read_u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.read()?))
    }

    pub fn read_u32_from(&mut self, ptr: u64) -> Result<u32, String> {
        self.cursor_to(ptr)?;
        self.read_u32()
    }

    pub fn read_u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.read()?))
    }

    pub fn read_u64_from(&mut self, ptr: u64) -> Result<u64, String> {
        self.cursor_to(ptr)?;
        self.read_u64()
    }

    pub fn read_dyn(&mut self, size: usize) -> Result<Vec<u8>, String> {
        let mut res = vec![0; size];
        self.file.read(res.as_mut_slice()).map_err(|err| err.to_string())?;
        Ok(res)
    }

}