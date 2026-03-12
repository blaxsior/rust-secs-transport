use crate::transport::secs1::config::Secs1TransportConfig;


///
/// Secs-I Block transfer 수준의 통신을 구현
/// 
pub struct Secs1Link {
    config: Secs1TransportConfig,
}