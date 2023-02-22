use std::collections::HashMap;
use std::net::TcpStream;
use std::process::Command;
use std::time::Duration;
use regex::Regex;
use serde::{Serialize,Deserialize};
use ssh_rs::{LocalSession, LocalShell, ssh};
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

pub fn database_check(host:&mut RemoteHost,thing:&str){
    let shell_command = format!("civm_db_check --runno={}",thing);
    let str = r#"(\{"runno_exists":.*?\}\})"#;
    let re = Regex::new(str).expect("invalid regex");
    let cap = host.run_and_listen(&shell_command,re).expect("response not captured");
    println!("capture = {}",cap);
}

pub enum Host {
    Local,
    Remote(RemoteHost)
}


impl Host {
    pub fn submit_request(&mut self,req:&Request) -> Response {
        match self {
            Host::Remote(remote_host) => remote_host.submit_request(req),
            Host::Local => {
                match process_request(&req.to_json()) {
                    Err(e) => Response::Error(e),
                    Ok(stat) => Response::Success(stat)
                }
            }
        }
    }

    pub fn civm_db_exists(&mut self, things:&Vec<String>) -> HashMap<String,DBStatus> {
        let cmd = "civm_db_check";
        let args:Vec<String> = things.iter().map(|thing|format!("--exists={}",thing)).collect();
        let re = Regex::new(r#"(\{"exists":.*?\}\})"#).expect("invalid regex");
        let cap = match self {
            Host::Remote(computer) => {
                println!("running db check remotely ...");
                let remote_cmd = format!("{} {}",cmd,args.join(" "));
                computer.run_and_listen(&remote_cmd,re).expect("no response captured")
            }
            Host::Local => {
                println!("running db check locally ...");
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

    pub fn civm_in_db(&mut self,things:&Vec<String>) -> bool {
        let results = self.civm_db_exists(things);
        let mut n_found = 0;
        for (_,stat) in &results {
            match stat {
                DBStatus::Found => {
                    n_found = n_found + 1;
                }
                _=> {}
            }
        }
        n_found == results.len()
    }
}

pub struct RemoteHost {
    hostname:String,
    user_name:String,
    port:u32,
    session:LocalSession<TcpStream>,
    shell:LocalShell<TcpStream>
}

#[derive(Debug)]
pub enum ConnectionError {
    UnableToConnect,
    UnableToStartShell,
    NoPublicKeysFound,
    UnableToInitialize
}

impl RemoteHost {
    pub fn new(hostname: &str, user_name: &str, port: u32) -> Result<Self, ConnectionError> {
        let ssh_dir = std::env::home_dir().expect("cannot get home dir").join(".ssh");
        let pub_keys = utils::find_files(&ssh_dir,"pub",true).ok_or(ConnectionError::NoPublicKeysFound)?;
        // println!("{:?}",pub_keys);
        // lazy search for a private key that works
        let mut session = pub_keys.iter().find_map(|key_file|{
            match ssh::create_session().username(user_name).private_key_path(&key_file.with_extension("")).connect(&format!("{}:{}", hostname, port)){
                Err(_) => None,
                Ok(thing) => Some(thing.run_local())
            }
        }).ok_or(ConnectionError::UnableToConnect)?;
        let shell = session.open_shell().map_err(|_| ConnectionError::UnableToStartShell)?;
        Ok(RemoteHost {
            hostname: hostname.to_string(),
            user_name: user_name.to_string(),
            port,
            session,
            shell,
        })
    }

    // attempt the run the server binary on the host. we should get a response from the server, otherwise
    // we will consider it a connection error
    pub fn check_for_server_bin(&mut self) -> Result<(),ConnectionError> {
        println!("\tchecking for server binary ...");
        let shell_cmd = "pipe_status_server";
        let re = Regex::new(r"pipe_status_server:=(.*)").expect("incorrect regular expression");
        match self.run_and_listen(shell_cmd,re) {
            None => Err(ConnectionError::UnableToInitialize),
            Some(_) => Ok(())
        }
    }

    pub fn run_and_listen(&mut self, shell_command:&str, regex_capture_pattern:Regex) -> Option<String> {
        let cmd = shell_command.to_owned() + "\n";
        self.shell.write(cmd.as_bytes()).expect(&format!("unable to write to shell on {}", self.hostname));
        let mut string_response = String::new();
        let str = loop {
            let byte_chunk = match self.shell.read(){
                Err(_) => {
                    println!("shell read timeout!");
                    break None
                } ,
                Ok(bytes) => bytes
            };
            let string_buffer = String::from_utf8(byte_chunk).unwrap();
            string_response.push_str(&string_buffer);
            // check that string_response contains the json
            let txt = string_response.as_str();
            let capture = regex_capture_pattern.captures(txt);
            match capture {
                Some(cap) => {
                    break Some(cap.get(1).expect("no group captured").as_str());
                }
                None => {

                }
            }
        };
        match str {
            Some(str) => Some(str.to_owned()),
            None => None
        }
    }

    pub fn submit_request(&mut self,request:&Request) -> Response {
        //println!("\t{:#?}",&request);
        let req_string = serde_json::to_string(request).expect("unable to serialize request");
        let command_string = format!("pipe_status_server --request-string='{}'",req_string);
        println!("\tcommand = {:}",&command_string);
        let re = Regex::new(r"\|\|(.*)\|\|").expect("incorrect regular expression");
        let json_response = self.run_and_listen(&command_string,re);
        println!("\tresponse = {:}",json_response.clone().unwrap());
        match json_response {
            Some(json) => serde_json::from_str(&json).expect("cannot deserialize response"),
            None => Response::Error(ServerError::RequestParse)
        }
    }
}
