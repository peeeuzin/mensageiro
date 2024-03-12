use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct Request {
    #[serde(rename = "type")]
    pub kind: Type,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    #[serde(rename = "type")]
    pub kind: Type,
    pub path: Option<String>,
    pub status: Status,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Status {
    /// The request was successful.
    Ok,
    /// The request could not be parsed. Client error response.
    Unparseable,
    /// The request was not understood. Client error response.
    BadRequest,
}

#[derive(Serialize, Debug)]
pub struct Error {
    message: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum Type {
    Publish,
    Subscribe,
    Unsubscribe,
    Message,
    Generic,
}
