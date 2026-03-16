use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::transport::error::SecsTransportError;

const WITHOUT_MSB: u8 = 0x7F;
const MSB_ONLY: u8 = 0x80;

///
/// SECS-I Block Transfer Protocol 중 사용되는 구조체
///
pub struct Secs1Block {
    pub header: Secs1BlockHeader,
    pub data: Vec<u8>,
}

///
/// SECS-I block header을 표현하는 구조체
///
pub struct Secs1BlockHeader {
    pub rbit: bool,
    pub device_id: u16,

    pub wbit: bool,
    pub stream: u8,
    pub function: u8,

    pub ebit: bool,
    pub block_no: u16,

    pub system_bytes: u32,
}

impl Secs1BlockHeader {
    pub fn to_bytes(&self) -> [u8; 10] {
        let mut h = [0u8; 10];

        h[0] = ((self.rbit as u8) << 7) | ((self.device_id >> 8) as u8 & WITHOUT_MSB);
        h[1] = self.device_id as u8;

        h[2] = ((self.wbit as u8) << 7) | (self.stream & WITHOUT_MSB);
        h[3] = self.function;

        h[4] = ((self.ebit as u8) << 7) | ((self.block_no >> 8) as u8 & WITHOUT_MSB);
        h[5] = self.block_no as u8;

        h[6..10].copy_from_slice(&self.system_bytes.to_be_bytes());

        h
    }
}

impl TryFrom<[u8; 10]> for Secs1BlockHeader {
    type Error = SecsTransportError;

    fn try_from(h: [u8; 10]) -> Result<Self, Self::Error> {
        Ok(Self {
            rbit: h[0] & MSB_ONLY != 0,
            device_id: u16::from_be_bytes([h[0] & WITHOUT_MSB, h[1]]),

            wbit: h[2] & MSB_ONLY != 0,
            stream: h[2] & WITHOUT_MSB,
            function: h[3],

            ebit: h[4] & MSB_ONLY != 0,
            block_no: u16::from_be_bytes([h[4] & WITHOUT_MSB, h[5]]),

            system_bytes: u32::from_be_bytes([h[6], h[7], h[8], h[9]]),
        })
    }
}

impl Secs1Block {
    pub fn checksum(&self) -> u16 {
        self.header
            .to_bytes()
            .iter()
            .chain(self.data.iter())
            .fold(0u16, |acc, b| acc.wrapping_add(*b as u16))
    }

    pub fn verify_checksum(&self, expected: u16) -> bool {
        self.checksum() == expected
    }

    /// bytes 배열로 변환
    pub fn to_bytes(&self) -> Vec<u8> {
        let header = self.header.to_bytes();

        let mut buf = Vec::with_capacity(header.len() + self.data.len());

        buf.extend_from_slice(&header);
        buf.extend_from_slice(&self.data);

        buf
    }
}

impl TryFrom<&[u8]> for Secs1Block {
    type Error = SecsTransportError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 10 || value.len() > 254 {
            return Err(SecsTransportError::BlockInvalid);
        }

        let raw_header: [u8; 10] = value[0..10]
            .try_into()
            .map_err(|_| SecsTransportError::BlockInvalid)?;

        let header = Secs1BlockHeader::try_from(raw_header)?;

        Ok(Self {
            header,
            data: value[10..].to_vec(),
        })
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
