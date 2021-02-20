/// Error type for OSC operations.
///
/// An error type for the errors that may happen while sending or receiving messages over an OSC
/// socket.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// IO error
    #[error("IO error")]
    Io(#[from] std::io::Error),
    /// OSC decode error
    #[error("Decode OSC packet failed")]
    Osc(rosc::OscError),
}

impl From<rosc::OscError> for Error {
    fn from(error: rosc::OscError) -> Self {
        Self::Osc(error)
    }
}

/// Result type for OSC operations.
pub type Result<T> = std::result::Result<T, Error>;
