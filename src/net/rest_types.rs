//! REST API response types.

use serde::Serialize;

/// Standard REST response envelope.
#[derive(Debug, Serialize)]
pub struct RestResponse<T: Serialize> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
}

/// Standard REST error response.
#[derive(Debug, Serialize)]
pub struct RestError {
    pub error: String,
    pub code: u16,
}

impl RestError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            error: msg.into(),
            code: 404,
        }
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            error: msg.into(),
            code: 400,
        }
    }
}
