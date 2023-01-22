use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response{
    status: u16,
    message: String,
    value: String,
}

impl Response{
    pub fn new(status: u16, message: &str, value: &str) -> Self{
        Self{
            status,
            message: message.to_string(),
            value: message.to_string(),
        }
    }
}
