pub mod bitstream;
pub mod block_writer;
pub mod block_reader;

pub const MAGIC: &[u8; 4] = b"TTH1";
pub const DEFAULT_BLOCK_SAMPLES: usize = 256;
pub const MICRO_BLOCK_SAMPLES: usize = 128;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StreamDataType {
    Bytes = 0,
    I64 = 1,
    F64 = 2,
}

impl StreamDataType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Bytes),
            1 => Some(Self::I64),
            2 => Some(Self::F64),
            _ => None,
        }
    }
}
