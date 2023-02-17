use serde::{Serialize,Deserialize};
use crate::request::ServerError;

#[derive(Debug,Serialize,Deserialize,Clone)]
pub enum StatusType {
    InProgress(f32),
    NotStarted,
    Complete,
    Invalid(ServerError),
}

impl StatusType {
    pub fn to_float(&self) -> f32 {
        match &self {
            StatusType::Invalid(_) => panic!("invalid status detected!"),
            StatusType::Complete => 1.0,
            StatusType::NotStarted => 0.0,
            StatusType::InProgress(progress) => *progress
        }
    }
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Status {
    pub label:String,
    pub progress:StatusType,
    pub children:Vec<Status>
}