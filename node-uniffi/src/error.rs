use thiserror::Error;

/// Result type alias for DATA Network node operations.
pub type Result<T, E = DataNetworkError> = std::result::Result<T, E>;

/// Represents errors that can occur in the DATA Network node.
#[derive(Error, Debug, uniffi::Error)]
pub enum DataNetworkError {
    /// Error returned when trying to perform operations on a node that isn't running.
    #[error("Node is not running")]
    NodeNotRunning,

    /// Error returned when trying to start a node that's already running.
    #[error("Node is already running")]
    AlreadyRunning,

    /// Error returned when configuration is invalid.
    #[error("Invalid configuration: {msg}")]
    InvalidConfig { msg: String },

    /// Error returned when an RPC input is invalid or malformed.
    #[error("Invalid request: {msg}")]
    InvalidRequest { msg: String },

    /// Error returned when the light client operation fails.
    #[error("Client error: {msg}")]
    Client { msg: String },

    /// Error returned when a value cannot be serialized.
    #[error("Serialization error: {msg}")]
    Serialization { msg: String },
}

impl DataNetworkError {
    pub(crate) fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig { msg: msg.into() }
    }

    pub(crate) fn invalid_request(msg: impl Into<String>) -> Self {
        Self::InvalidRequest { msg: msg.into() }
    }

    pub(crate) fn client(msg: impl Into<String>) -> Self {
        Self::Client { msg: msg.into() }
    }

    pub(crate) fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization { msg: msg.into() }
    }
}
