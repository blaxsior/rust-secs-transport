use tokio_util::{bytes::{Buf, BufMut, BytesMut}, codec::{Decoder, Encoder}};
use crate::transport::secs1::message::Secs1Message;



fn calculate_checksum(body: &[u8]) -> u16 {
    body.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16))
}

/// Secs1Message를 처리하는 구조체
pub struct Secs1Codec;

impl Decoder for Secs1Codec {
    type Item = Secs1Message;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // 1. 최소 길이 확인 (L:1 + H:10 + C:2 = 13)
        if src.len() < 13 { return Ok(None); }

        // 2. 전체 프레임 길이 계산
        let payload_len = src[0] as usize; // Header + Data
        let total_len = payload_len + 3;   // Length(1) + Payload + Checksum(2)

        if src.len() < total_len { return Ok(None); }

        // 3. 체크섬 검증 (간단하게 합계만 확인)
        let body = &src[1..total_len - 2];
        let checksum = u16::from_be_bytes([src[total_len-2], src[total_len-1]]);
        let actual_sum: u16 = calculate_checksum(body);

        if checksum != actual_sum {
            src.advance(total_len); // 깨진 데이터는 버림
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Checksum Error"));
        }

        // 4. 데이터 추출 및 자르기
        let _len_byte = src.get_u8();
        
        let mut header = [0u8; 10];
        src.copy_to_slice(&mut header);
        
        let data = src.split_to(payload_len - 10).to_vec();
        let _checksum_bytes = src.get_u16();

        Ok(Some(Secs1Message { header, data }))
    }
}

impl Encoder<Secs1Message> for Secs1Codec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Secs1Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let payload_len = 10 + item.data.len();
        dst.reserve(payload_len + 3);

        dst.put_u8(payload_len as u8); // Length
        let start = dst.len();
        dst.put_slice(&item.header);   // Header
        dst.put_slice(&item.data);     // Data
        let checksum: u16 = calculate_checksum(&dst[start..]);
        dst.put_u16(checksum);         // Checksum

        Ok(())
    }
}