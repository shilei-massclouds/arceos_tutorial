use crate::{DeviceTreeResult, DeviceTreeError};

pub trait SliceRead {
    fn read_be_u32(&self, pos: usize) -> DeviceTreeResult<u32>;
    fn read_be_u64(&self, pos: usize) -> DeviceTreeResult<u64>;
    fn read_bstring0(&self, pos: usize) -> DeviceTreeResult<&[u8]>;
    fn subslice(&self, start: usize, len: usize) -> DeviceTreeResult<&[u8]>;
}

impl<'a> SliceRead for &'a [u8] {
    fn read_be_u32(&self, pos: usize) -> DeviceTreeResult<u32> {
        // check size is valid
        if ! (pos+4 <= self.len()) {
            return Err(DeviceTreeError::SliceReadError)
        }

        Ok(
            (self[pos] as u32) << 24
            | (self[pos+1] as u32) << 16
            | (self[pos+2] as u32) << 8
            | (self[pos+3] as u32)
        )
    }
    fn read_be_u64(&self, pos: usize) -> DeviceTreeResult<u64> {
        let hi: u64 = self.read_be_u32(pos)?.into();
        let lo: u64 = self.read_be_u32(pos+4)?.into();
        Ok((hi << 32) | lo)
    }
    fn read_bstring0(&self, pos: usize) -> DeviceTreeResult<&[u8]> {
        let mut cur = pos;
        while cur < self.len() {
            if self[cur] == 0 {
                return Ok(&self[pos..cur])
            }
            cur += 1;
        }
        Err(DeviceTreeError::SliceReadError)
    }
    fn subslice(&self, start: usize, end: usize) -> DeviceTreeResult<&[u8]> {
        if ! (end < self.len()) {
            return Err(DeviceTreeError::SliceReadError)
        }
        Ok(&self[start..end])
    }
}
