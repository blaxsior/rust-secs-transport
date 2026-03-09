use std::{error::Error, fmt::Display};

///
/// SecsTransport 처리 시 예외
///
#[derive(Debug)]
pub struct SecsTransportError {
    pub err_kind: SecsTransportErrKind,
    pub description: String,
}

#[derive(Debug)]
pub enum SecsTransportErrKind {
    ConnectionFailed,
}

impl Error for SecsTransportError {
}

impl Display for SecsTransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description)
    }
}
