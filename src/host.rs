use std::net::TcpStream;
use std::time::Duration;
use regex::Regex;
use ssh_rs::{LocalSession, LocalShell, ssh};
use crate::request::{Request, Response, ServerError};

pub struct Host {
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
}

impl Host {
    pub fn new(hostname: &str, user_name: &str, port: u32) -> Result<Self, ConnectionError> {
        let private_key = std::env::home_dir().expect("cannot get home dir").join(".ssh").join("id_rsa");
        let mut session = ssh::create_session()
            .username(user_name)
            .private_key_path(private_key)
            .connect(&format!("{}:{}", hostname, port)).map_err(|_| ConnectionError::UnableToConnect)?
            .run_local();
        let shell = session.open_shell().map_err(|_| ConnectionError::UnableToStartShell)?;

        Ok(Host {
            hostname: hostname.to_string(),
            user_name: user_name.to_string(),
            port,
            session,
            shell,
        })
    }

    // this just forces the shell to initialize to avoid delays later on
    pub fn check_for_bin(&mut self) {
        println!("initializing {} ...",self.hostname);
        self.shell.write(b"pipe_status_server\n").expect("cannot write to shell");

        let mut string_response = String::new();

        let re = Regex::new(r"pipe_status_server:=(.*)").expect("incorrect regular expression");

        let received = loop {
            let byte_chunk = match self.shell.read(){
                Err(_) => {
                    println!("shell read error!");
                    break None
                } ,
                Ok(bytes) => bytes
            };
            let string_buffer = String::from_utf8(byte_chunk).unwrap();
            string_response.push_str(&string_buffer);

            // check that string_response contains the json

            let txt = string_response.as_str();

            let capture = re.captures(txt);

            match capture {
                Some(cap) => {
                    break Some(cap.get(1).expect("no group captured").as_str());
                }
                None => {

                }
            }
        }.expect("failed to retrieve response from host");

        println!("recieved = {}",received);

    }

    pub fn run_and_listen(&mut self, shell_command:&str, regex_capture_pattern:Regex) -> Option<String> {
        self.shell.write(shell_command.as_bytes()).expect(&format!("unable to write to shell on {}", self.hostname));
        println!("wrote command to shell ...");
        let mut string_response = String::new();
        let str = loop {
            let byte_chunk = match self.shell.read(){
                Err(_) => {
                    println!("shell read error!");
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
        let req_string = serde_json::to_string(request).expect("unable to serialize request");
        let command_string = format!("pipe_status_server --request-string='{}'\n",req_string);
        let re = Regex::new(r"\|\|(.*)\|\|").expect("incorrect regular expression");
        let json_response = self.run_and_listen(&command_string,re);
        match json_response {
            Some(json) => serde_json::from_str(&json).expect("cannot deserialize response"),
            None => Response::Error(ServerError::RequestParse)
        }
    }

    // pub fn submit_request(&mut self,request:&Request) -> Response {
    //
    //     let req_string = serde_json::to_string(request).expect("unable to serialize request");
    //
    //     let command_string = format!("pipe_status_server --request-string='{}'\n",req_string);
    //     self.shell.write(command_string.as_bytes()).expect(&format!("unable to write to shell on {}",self.hostname));
    //
    //     let mut string_response = String::new();
    //
    //     let re = Regex::new(r"\|\|(.*)\|\|").expect("incorrect regular expression");
    //
    //     let json = loop {
    //         let byte_chunk = match self.shell.read(){
    //             Err(_) => {
    //                 println!("shell read error!");
    //                 break None
    //             } ,
    //             Ok(bytes) => bytes
    //         };
    //         let string_buffer = String::from_utf8(byte_chunk).unwrap();
    //         string_response.push_str(&string_buffer);
    //
    //         // check that string_response contains the json
    //
    //         let txt = string_response.as_str();
    //
    //         let capture = re.captures(txt);
    //
    //         match capture {
    //             Some(cap) => {
    //                 break Some(cap.get(1).expect("no group captured").as_str());
    //             }
    //             None => {
    //
    //             }
    //         }
    //     };
    //
    //     match json {
    //         Some(json) => serde_json::from_str(json).expect("cannot deserialize response"),
    //         None => Response::Error(ServerError::RequestParse)
    //     }
    //
    // }

}