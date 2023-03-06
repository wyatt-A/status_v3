use std::collections::HashMap;
use std::path::{Path, PathBuf};
use clap::Parser;
use clap;
use ssh_config::SSHConfig;
use dirs;
use status_v3::host::{ConnectionError, Host, RemoteHost};
use status_v3::pipe::{ConfigCollection, ConfigCollectionError};
use status_v3::args::{Args, Action, ClientArgs, GenTemplateArgs};
use status_v3::status::{Status, StatusType};
use civm_rust_utils as utils;


pub const DEFAULT_PIPE_CONFIG_DIR:&str = "status_configs";

pub enum ClientError {
    ConnectionError(ConnectionError)
}

fn main(){
    let args:Args = Args::parse();
    match args.action {
        Action::Check(client_args) => {
            match run_client(&client_args) {
                Ok(stat) => {
                    let txt = serde_json::to_string_pretty(&stat).expect("cannot convert to string");
                    println!("{}",txt);
                }
                Err(_) => {
                    return
                }
            }
        },
        Action::GenTemplates(template_args) => gen_templates(template_args)
    };
}

fn gen_templates(args:GenTemplateArgs) {
    match args.directory {
        Some(dir) => ConfigCollection::generate_templates(&dir),
        None => {
            let home_dir:PathBuf = dirs::home_dir().expect("cannot get home directory!");
            let dir = home_dir.join(DEFAULT_PIPE_CONFIG_DIR);
            ConfigCollection::generate_templates(&dir)
        }
    }
}

fn run_client(args:&ClientArgs) -> Result<Status,()>{

    //let this_exe = std::env::current_exe().expect("cannot determine this program!");

    let home_dir:PathBuf = dirs::home_dir().expect("cannot get home directory!");
    let ssh_dir = home_dir.join(".ssh");
    let ssh_config = ssh_dir.join("config");
    let this_host = utils::computer_name();

    let mut this_user = std::env::var("USER").expect("unable to get environment variable");
    if this_user.is_empty(){
        this_user = std::env::var("USERNAME").expect("unable to get environment variable");
    }

    println!("loading config files ...");
    let pipe_conf_dir = match &args.pipe_configs {
        Some(config_path) => config_path.clone(),
        None => {
            // lets first look in pipeline settings for configs. If the variable isn't set or the dir doesn't exist, resort to home directory
            if let Ok(wks_settings_dir) = std::env::var("WKS_SETTINGS") {
                let conf_dir = Path::new(wks_settings_dir.as_str()).join(DEFAULT_PIPE_CONFIG_DIR);
                if !conf_dir.exists() {
                    home_dir.join(DEFAULT_PIPE_CONFIG_DIR)
                }else {
                    conf_dir
                }
            }else {
                home_dir.join(DEFAULT_PIPE_CONFIG_DIR)
            }
        }
    };
    let conf_col = match ConfigCollection::from_dir(&pipe_conf_dir) {
        Err(error) => {
            match error {
                ConfigCollectionError::NoConfigsFound => {
                    println!("no config files found in {:?}.\n\
                    Please specify a correct directory containing configurations with --pipe-configs=/some/path\n\
                    or generate some templates with pipe_status gen-templates",pipe_conf_dir);
                    Err(())?
                }
                ConfigCollectionError::ConfigParse(file,toml_error) => {
                    println!("an error occurred when parsing config file: {:?}\n{:?}",file,toml_error);
                    Err(())?
                }
            }
        }
        Ok(conf_col) => conf_col
    };

    println!("resolving pipeline hosts ...");
    // get list of hosts from last_pipe
    let needed_hosts = conf_col.required_servers(&args.last_pipeline);

    // parse big_disk option
    let big_disks = match args.parse_big_disk(){
        Ok(big_disks) => big_disks,
        Err(_) => {
            println!("big-disk must be of the form:\ncomputer:/some/path");
            Err(())?
        }
    };

    println!("checking ssh configurations ...");
    // load ssh config and check for existence
    if !ssh_config.exists(){
        println!("no ssh config found! You need to set this up first!\n\
        we are expecting something like this in your .ssh/config file...\n\
        Host <computer_alias>\n\
            HostName <computer_name>\n\
            User <username_for_computer>"
        );
        Err(())?
    }

    // load ssh config file and parse it to a usable type
    let config_str = utils::read_to_string(&ssh_config,Some(""));
    let ssh_conf = SSHConfig::parse_str(&config_str).unwrap();

    // connect to servers
    println!("connecting to remote hosts ...");
    let mut host_connections = HashMap::<String, Host>::new();
    for host in &needed_hosts {
        let host_config = ssh_conf.query(host);
        let username = host_config.get("User");

        let username = match username {
            None => this_user.as_str().to_string(),
            Some(user) => user.to_string()
        };


        // check if the needed host is this host
        match host == this_host.as_str(){
            true => {
                host_connections.insert(host.to_string(), Host::Local);
            }
            false => {
                println!("\t{}@{}", &username.as_str(), host);
                match RemoteHost::new(host, username.as_str(), 22) {
                    Err(conn_error) =>{
                        match conn_error {
                            ConnectionError::NoPublicKeysFound => {
                                println!("no ssh public keys found in {:?}\nRun ssh-keygen and make sure you have password-less access to {}", ssh_dir, host);
                                Err(())?;
                            }
                            ConnectionError::UnableToConnect => {
                                println!("unable to connect to {}. Make sure you have password-less access!\nYou may need to run ssh-copy-id {}@{}", host, username, host);
                                Err(())?
                            }
                            ConnectionError::UnableToStartShell => {
                                println!("unable to start a shell on {}.", host);
                                Err(())?
                            }
                            _=> {}
                        }
                    } ,
                    Ok(mut connected_host) => {
                        match connected_host.check_for_server_bin() {
                            Err(_) => {
                                println!("unable to successfully talk to status server on {}.", host);
                                Err(())?
                            }
                            Ok(_) => {
                                host_connections.insert(host.to_string(), Host::Remote(connected_host));
                            }
                        }
                    }
                };
            }
        }
    }

    let mut status = conf_col.pipe_status(&args.last_pipeline, &args, &mut host_connections, &this_host, &big_disks);
    /*
    for s in &status.children{
        let txt = serde_json::to_string_pretty(s).expect("cannot convert to string");
        println!("{}",txt);
    }
    */

    // do something useful with status
    // Lets start by limiting prints
    match status.progress {
        StatusType::Complete => {
            println!("\n\n{} Complete",status.label);
            //status.children = vec![];
        }
        StatusType::InProgress(_) => {
            let mut partial_status=status.clone();
            let mut children_with_progress = vec![];
            let mut stage_number=0;
            for s in &partial_status.children {
                stage_number+=1;
                match s.progress {
                    //StatusType::Complete|StatusType::InProgress(_)|StatusType::Invalid(_) =>{
                    StatusType::Complete|StatusType::InProgress(_)=>{
                        children_with_progress.push(s.clone());
                    }
                    _ => {}
                }
            }
            partial_status.children=children_with_progress;
            if ! args.print_all {
                status = partial_status;
            }
        }
        StatusType::Invalid(_)|StatusType::NotStarted => {
        }
    }

    Ok(status)
    // debuggy print pretty
    //println!("{:#?}",status);
    // dump all in "portable" json


}
