use std::collections::HashMap;
use std::path::PathBuf;
use clap::{Parser};
use clap;
use ssh_config::{ConfigValue, SSHConfig};
use dirs;
use status_v3::host::{ConnectionError, Host};
use status_v3::pipe::{ConfigCollection, ConfigCollectionError};
use whoami;
use status_v3::args::{Args, Action, ClientArgs, GenTemplateArgs};


pub const DEFAULT_PIPE_CONFIG_DIR:&str = ".pipe_config";


fn main(){
    let args:Args = Args::parse();
    match args.action {
        Action::Check(client_args) =>  run_client(&client_args),
        Action::GenTemplates(template_args) => gen_templates(template_args)
    }
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

fn run_client(args:&ClientArgs){

    //let this_exe = std::env::current_exe().expect("cannot determine this program!");


    let home_dir:PathBuf = dirs::home_dir().expect("cannot get home directory!");
    let ssh_dir = home_dir.join(".ssh");
    let ssh_config = ssh_dir.join("config");
    let this_host = utils::computer_name();


    let mut this_user = std::env::var("USER").expect("unable to get environment variable");
    if this_user.is_empty(){
        this_user = std::env::var("USERNAME").expect("unable to get environment variable");
    }

    //let this_user = whoami::username();

    println!("you are: {}",this_user);

    println!("loading config files ...");

    let pipe_conf_dir = args.pipe_configs.clone().unwrap_or(
        home_dir.join(DEFAULT_PIPE_CONFIG_DIR)
    );

    let conf_col = match ConfigCollection::from_dir(&pipe_conf_dir) {
        Err(error) => {
            match error {
                ConfigCollectionError::NoConfigsFound => {
                    println!("no config files found in {:?}.\n\
                    Please specify with --pipe-configs=/some/path\n\
                    or generate some templates with pipe_status gen-templates",pipe_conf_dir);
                    return
                }
                ConfigCollectionError::ConfigParse(file,toml_error) => {
                    println!("an error occurred when parsing config file: {:?}\n{:?}",file,toml_error);
                    return
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
            return
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
        return
    }

    // load ssh config file and parse it to a usable type
    let config_str = utils::read_to_string(&ssh_config,"");
    let ssh_conf = SSHConfig::parse_str(&config_str).unwrap();

    // println!("checking for {:?} in .ssh/config", needed_hosts);
    // // check for a config for each server
    // for host in &needed_hosts {
    //     let server_config = ssh_conf.query(host);
    //     if server_config.is_empty(){
    //         println!("unable to find a ssh config for {}.\nPlease add a config for {} in {:?} like the following...\nHost {}\n   HostName {}\n   User your_username_on_{}"
    //         ,host,host,ssh_config,host,host,host);
    //         return
    //     }
    // }

    // connect to servers
    println!("connecting to remote hosts ...");
    let mut ssh_connections = HashMap::<String,Host>::new();
    for host in &needed_hosts {
        let host_config = ssh_conf.query(host);
        let username = host_config.get("User");

        let username = match username {
            None => this_user.as_str().to_string(),
            Some(user) => user.to_string()
        };

        match Host::new(host, username.as_str(), 22) {
            Err(conn_error) =>{
                match conn_error {
                    ConnectionError::NoPublicKeysFound => {
                        println!("no ssh public keys found in {:?}\nRun ssh-keygen and make sure you have password-less access to {}", ssh_dir, host);
                        return;
                    }
                    ConnectionError::UnableToConnect => {
                        println!("unable to connect to {}. Make sure you have password-less access!\nYou may need to run ssh-copy-id {}@{}", host, username, host);
                        return
                    }
                    ConnectionError::UnableToStartShell => {
                        println!("unable to start a shell on {}.", host);
                        return
                    }
                    _=> {}
                }
            } ,
            Ok(mut connected_host) => {
                match connected_host.check_for_server_bin() {
                    Err(_) => {
                        println!("unable to successfully talk to status server on {}.", host);
                        return
                    }
                    Ok(_) => {
                        ssh_connections.insert(host.to_string(), connected_host);
                    }
                }
            }
        };
    }

    let (status,prog) = conf_col.pipe_status(&args.last_pipeline,&args,&mut ssh_connections,&this_host,&big_disks);

    for s in &status{
        let txt = serde_json::to_string_pretty(s).expect("cannot convert to string");
        println!("{}",txt);
    }


    // do something useful with status'



}


