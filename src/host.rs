use std::collections::HashMap;
use std::process::Command;
use regex::Regex;
use serde::{Serialize,Deserialize};
use crate::host_connection::{PipeStatusHost, RemoteHostConnection};
use crate::request::{Request, Response, ServerError};
use crate::server::process_request;

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DbResponse {
    exists:HashMap<String,i32>
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum DBStatus {
    Found,
    NotFound,
    Unknown,
}

impl DbResponse {
    pub fn to_hash(&self) -> HashMap<String,DBStatus> {
        let mut output = HashMap::<String,DBStatus>::new();
        for (thing,stat) in &self.exists {
            let db_stat = match stat {
                1 => DBStatus::Found,
                0 => DBStatus::NotFound,
                -1 => DBStatus::Unknown,
                _=> DBStatus::Unknown
            };
            output.insert(thing.clone(),db_stat);
        }
        output
    }
}

pub fn database_check(host:&mut RemoteHostConnection,thing:&str){
    let shell_command = format!("civm_db_check --runno={}",thing);
    let str = r#"(\{"runno_exists":.*?\}\})"#;
    let re = Regex::new(str).expect("invalid regex");
    let cap = host.run_and_listen(&shell_command,re).expect("response not captured");
    println!("capture = {}",cap);
}


impl PipeStatusHost {
    pub fn submit_request(&mut self,req:&Request) -> Response {
        match self {
            PipeStatusHost::Remote(remote_host) => submit_request(remote_host, req),
            PipeStatusHost::Local => {
                match process_request(&req.to_json()) {
                    Err(e) => Response::Error(e),
                    Ok(stat) => Response::Success(stat)
                }
            }
        }
    }



}

pub fn submit_request(remote_host:&mut RemoteHostConnection,request:&Request) -> Response {
    let req_string = serde_json::to_string(request).expect("unable to serialize request");
    let command_string = format!("pipe_status_server --request-string='{}'",req_string);
    match remote_host.run_and_listen(&command_string,r"\|\|(.*)\|\|") {
        Ok(json_response) => serde_json::from_str(&json_response).expect("cannot deserialize response"),
        Err(error) => Response::Error(ServerError::HostConnectionError(error))
    }
}

pub fn civm_db_exists(remote_host:&mut RemoteHostConnection, things:&Vec<String>) -> HashMap<String,DBStatus> {
    let cmd = "civm_db_check";
    let regex_capture_pattern = r#"(\{"exists":.*?\}\})"#;
    let args:Vec<String> = things.iter().map(|thing|format!("--exists={}",thing)).collect();
    let cap = match remote_host {
        PipeStatusHost::Remote(computer) => {
            println!("running db check remotely ...");
            let remote_cmd = format!("{} {}",cmd,args.join(" "));
            remote_host.run_and_listen(&remote_cmd,regex_capture_pattern).expect("no response captured")
        }
        PipeStatusHost::Local => {
            println!("running db check locally ...");
            let re = Regex::new(regex_capture_pattern).expect("invalid regex");
            let o = Command::new(cmd).args(&args).output().expect("failed to launch db check");
            let r = String::from_utf8(o.stdout.clone()).unwrap();
            re.captures(&r).expect("command response not matched").get(1).expect("command response").as_str().to_string()
        }
    };

    let dbr:DbResponse = serde_json::from_str(&cap).expect("unable to deserialize database response");
    let h = dbr.to_hash();
    println!("{:?}",h);
    h
}

#[derive(Debug)]
pub enum ConnectionError {
    UnableToConnect,
    UnableToStartShell,
    NoPublicKeysFound,
    UnableToInitialize
}
