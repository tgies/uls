//! API response types.

use serde::Serialize;

/// Paginated list response envelope.
#[derive(Debug, Serialize)]
pub struct ListResponse<T: Serialize> {
    pub data: Vec<T>,
    pub count: usize,
    pub limit: usize,
    pub offset: usize,
}

impl<T: Serialize> ListResponse<T> {
    pub fn new(data: Vec<T>, limit: usize, offset: usize) -> Self {
        let count = data.len();
        Self {
            data,
            count,
            limit,
            offset,
        }
    }
}
