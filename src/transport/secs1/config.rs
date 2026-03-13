use std::time::Duration;

use crate::transport::ConnectionMode;

pub struct DeviceId(pub u16);
/// 
/// SECS-I 통신 구성 시 사용하는 설정
/// 
pub struct Secs1TransportConfig {
    /// 장치 식별자. 통신 장치의 식별 번호
    pub device_id: DeviceId,
    /// 통신 모드(ACTIVE / PASSIVE)
    pub mode: ConnectionMode,
    /// inter character timeout(ms): length byte ~ 2nd checksum byte
    pub t1_timeout: Duration,
    /// protocol timeout(ms): ENQ ~ EOT / EOT ~ length / 2nd checksum ~ any char
    pub t2_timeout: Duration,
    pub t3_timeout: Duration,
    pub t4_timeout: Duration,
    pub t2_rty: u8,
}