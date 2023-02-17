use std::collections::HashMap;
use std::path::PathBuf;
use clap::Parser;
use clap;
use ssh_config::SSHConfig;
use dirs;
use status_v3::host::{ConnectionError, PipeStatusHost, RemoteHost};
use status_v3::pipe::{ConfigCollection, ConfigCollectionError};
use status_v3::args::{Args, Action, ClientArgs, GenTemplateArgs};
use status_v3::host_connection::{connect_to_hosts, HostConnectionError, RemoteHostConnection};


struct BatchCheck {
    pipe_name:String,
    base_runno_list:Vec<String>
}



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
    let this_host = utils::computer_name();

    // let mut this_user = std::env::var("USER").expect("unable to get environment variable");
    // if this_user.is_empty(){
    //     this_user = std::env::var("USERNAME").expect("unable to get environment variable");
    // }

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

    // connect to servers
    println!("connecting to remote hosts ...");
    let mut host_connections = connect_to_hosts(&needed_hosts).or_else(|error|{
        println!("unable to connect to host {:?}",error);
        return
    });

    // see that the servers' executable is accessible
    for (_, connection) in host_connections.iter_mut() {
        if let PipeStatusHost::Remote(remote_connection) = connection {
            if let Err(error) = check_for_server_bin(remote_connection) {
                println!("unable to find server binary {:?}", error);
                return
            }
        }
    }


    let (status,prog) = conf_col.pipe_status(&args.last_pipeline, &args, &mut host_connections, &this_host, &big_disks);

    for s in &status{
        let txt = serde_json::to_string_pretty(s).expect("cannot convert to string");
        println!("{}",txt);
    }

    // do something useful with status'
}

pub fn check_for_server_bin(remote_host:&mut RemoteHostConnection) -> Result<(),HostConnectionError> {
    println!("checking for server binary on {} ...",remote_host.hostname);
    let shell_cmd = "pipe_status_server";
    remote_host.run_and_listen(shell_cmd,r"pipe_status_server:=(.*)")?;
    Ok(())
}