#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct BigEndianU32(u32);

impl BigEndianU32 {
    pub fn get(self) -> u32 {
        self.0
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(BigEndianU32(u32::from_be_bytes(bytes.get(..4)?.try_into().unwrap())))
    }
}

