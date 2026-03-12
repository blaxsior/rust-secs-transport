use num_enum::{IntoPrimitive, TryFromPrimitive};

const WITHOUT_MSB: u8 = 0x7F;
const MSB_ONLY: u8 = 0x80;

///
/// SECS-I Block Transfer Protocol 중 사용되는 구조체
/// 
pub struct Secs1Block {
    pub header: [u8; 10],
    pub data: Vec<u8>,
}

impl Secs1Block {
    pub fn rbit(&self) -> u8 {
        self.header[0] & MSB_ONLY
    }

    pub fn wbit(&self) -> u8 {
        self.header[2] & MSB_ONLY
    }

    pub fn ebit(&self) -> u8 {
        self.header[4] & MSB_ONLY
    }

    pub fn device_id(&self) -> u16 {
        u16::from_be_bytes([self.header[0] & WITHOUT_MSB, self.header[1]])
    }

    pub fn stream(&self) -> u8 {
        self.header[2] & WITHOUT_MSB
    }

    pub fn function(&self) -> u8 {
        self.header[3]
    }

    pub fn block_no(&self) -> u16 {
        u16::from_be_bytes([self.header[4] & WITHOUT_MSB, self.header[5]])
    }

    pub fn system_bytes(&self) -> u32 {
        u32::from_be_bytes([self.header[6],self.header[7],self.header[8],self.header[9]])
    }
}

/// Secs-I 통신 Block Transfer Protocol에서 사용되는 코드
#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum Secs1HandshakeCode {
    /// request to send
    ENQ = 0b00000101,
    /// ready to receive
    EOT = 0b00000100,
    /// correct reception
    ACK = 0b00000110,
    // incorrect reception
    NAK = 0b00010101,
}
