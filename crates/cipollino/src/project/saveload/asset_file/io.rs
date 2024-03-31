
use std::io::{self, Read, Seek, SeekFrom, Write};

use crate::project::saveload::asset_file::AssetFile;

impl AssetFile {

    pub fn cursor_to(&mut self, ptr: u64) -> Result<(), io::Error> {
        self.file.seek(SeekFrom::Start(ptr)).map(|_| ())
    }
    
    pub fn cursor_ptr(&mut self) -> Result<u64, io::Error> {
        self.file.seek(SeekFrom::Current(0))
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), io::Error> {
        self.file.write(data)?;
        Ok(())
    }

    pub fn write_u64(&mut self, val: u64) -> Result<(), io::Error> {
        self.write(&val.to_le_bytes())?;
        Ok(())
    }

    pub fn write_u64_to(&mut self, ptr: u64, val: u64) -> Result<(), io::Error> {
        self.cursor_to(ptr)?;
        self.write_u64(val)?;
        Ok(())
    }

    pub fn file_size(&mut self) -> Result<u64, io::Error> {
        self.file.seek(SeekFrom::End(0))
    }

    pub fn read<const N: usize>(&mut self) -> Result<[u8; N], io::Error> {
        let mut res = [0; N];
        self.file.read(&mut res)?;
        Ok(res)
    }

    pub fn read_u64(&mut self) -> Result<u64, io::Error> {
        Ok(u64::from_le_bytes(self.read()?))
    }

    pub fn read_dyn(&mut self, size: usize) -> Result<Vec<u8>, io::Error> {
        let mut res = vec![0; size];
        self.file.read(res.as_mut_slice())?;
        Ok(res)
    }

    pub fn read_u64_from(&mut self, ptr: u64) -> Result<u64, io::Error> {
        self.cursor_to(ptr)?;
        self.read_u64()
    }

}