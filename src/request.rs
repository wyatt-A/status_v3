use crate::stage::{FileCheckError, Stage};
use serde::{Serialize,Deserialize};
use crate::pipe::SubstitutionTable;
use crate::status::Status;

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Request {
    pub sub_table:SubstitutionTable,
    pub stage:Stage,
    pub big_disk:Option<String>,
    pub run_number_list:Vec<String>,
    pub base_runno:String,
}

impl Request {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).expect("cannot serialize request")
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum ServerError {
    Crash,
    BadResponse,
    RequestParse,
    BIGGUS_DISKUS_NotSet,
    FileCheckError(FileCheckError)
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum Response {
    Error(ServerError),
    Success(Status)
}