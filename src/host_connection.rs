use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::path::PathBuf;
use regex::Regex;
use ssh_config::SSHConfig;
use ssh_rs::{LocalSession, LocalShell, ssh};
use serde::{Serialize,Deserialize};
use crate::host::{ConnectionError};

#[derive(Debug,Serialize,Deserialize,Clone)]
pub enum HostConnectionError {
    LocalUserNameNotFound,
    SSHConfigNotFound,
    HomeDirectoryNotFound,
    SSHConfigParseError,
    NoPublicKeysFound,
    UnableToAuthenticate,
    UnableToStartShell,
    InvalidRegex,
    ResponseNotCaptured,
    Unknown,
}

pub enum PipeStatusHost {
    Local,
    Remote(RemoteHostConnection)
}

pub fn connect_to_hosts(needed_hosts:&HashSet<String>) -> Result<HashMap::<String, PipeStatusHost>,HostConnectionError>{

    use HostConnectionError::*;

    let home_dir:PathBuf = dirs::home_dir().ok_or(HomeDirectoryNotFound)?;
    let ssh_dir = home_dir.join(".ssh");
    let ssh_config = ssh_dir.join("config");
    let this_host = utils::computer_name();

    let this_user = utils::user_name().map_err(|_|LocalUserNameNotFound)?;

    if !ssh_config.exists(){
        return Err(SSHConfigNotFound)
    }

    let config_str = utils::read_to_string(&ssh_config,Some(""));
    let ssh_conf = SSHConfig::parse_str(&config_str).map_err(|_|SSHConfigParseError)?;

    let mut host_connections = HashMap::<String, PipeStatusHost>::new();
    for host in needed_hosts {
        let host_config = ssh_conf.query(host);
        let username = host_config.get("User");

        let username = match username {
            None => this_user.as_str().to_string(),
            Some(user) => user.to_string()
        };

        // check if the needed host is this host
        match host == this_host.as_str(){
            true => {
                host_connections.insert(host.to_string(), PipeStatusHost::Local);
            }
            false => {
                host_connections.insert(
                    host.clone(),
                    PipeStatusHost::Remote(
                        RemoteHostConnection::new(host, username.as_str(), 22).map_err(|e|{
                            match e {
                                ConnectionError::NoPublicKeysFound => NoPublicKeysFound,
                                ConnectionError::UnableToConnect => UnableToAuthenticate,
                                ConnectionError::UnableToStartShell => UnableToStartShell,
                                _ => Unknown
                            }
                        })?
                    )
                );
            }
        }
    }
    Ok(host_connections)
}

pub struct RemoteHostConnection {
    pub hostname:String,
    pub user_name:String,
    port:u32,
    session:LocalSession<TcpStream>,
    shell:LocalShell<TcpStream>
}


impl RemoteHostConnection {
    pub fn new(hostname: &str, user_name: &str, port: u32) -> Result<Self, ConnectionError> {
        let ssh_dir = std::env::home_dir().expect("cannot get home dir").join(".ssh");
        let pub_keys = utils::find_files(&ssh_dir,"pub",true).ok_or(ConnectionError::NoPublicKeysFound)?;
        // lazy search for a private key that works
        let mut session = pub_keys.iter().find_map(|key_file|{
            match ssh::create_session().username(user_name).private_key_path(&key_file.with_extension("")).connect(&format!("{}:{}", hostname, port)){
                Err(_) => None,
                Ok(thing) => Some(thing.run_local())
            }
        }).ok_or(ConnectionError::UnableToConnect)?;
        let shell = session.open_shell().map_err(|_| ConnectionError::UnableToStartShell)?;
        Ok(RemoteHostConnection {
            hostname: hostname.to_string(),
            user_name: user_name.to_string(),
            port,
            session,
            shell,
        })
    }

    pub fn run_and_listen(&mut self, shell_command:&str, regex_capture_pattern:&str) -> Result<String,HostConnectionError> {
        let regex_capture_pattern = Regex::new(regex_capture_pattern).map_err(|_|HostConnectionError::InvalidRegex)?;
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
            Some(str) => Ok(str.to_owned()),
            None => Err(HostConnectionError::ResponseNotCaptured)
        }
    }
}