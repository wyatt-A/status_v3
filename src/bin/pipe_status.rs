use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use serde::{Serialize, Deserialize};
use clap::{Command, Parser};
use clap;
use ssh_config::SSHConfig;
use dirs;
use status_v3::host::{ConnectionError, Host};
use status_v3::pipe::{ConfigCollection, ConfigCollectionError, PipeStatusConfig};
use status_v3::request::{Request, Response, ServerError};
use status_v3::status::{Status, StatusType};
use status_v3::args;
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

    let this_exe = std::env::current_exe().expect("cannot determine this program!");

    let home_dir:PathBuf = dirs::home_dir().expect("cannot get home directory!");

    let ssh_dir = home_dir.join(".ssh");
    let ssh_config = ssh_dir.join("config");

    let this_host = utils::computer_name();
    let this_user = whoami::username();

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
    let big_disks = match &args.big_disk {
        Some(args) => {
            let mut big_disks = HashMap::<String,String>::new();
            for arg in args {
                //let arg = arg.to_owned();
                let split:Vec<&str> = arg.split(":").collect();
                if split.len()  != 2 {
                    panic!("BIGGUS_DISKUS must contain a : for")
                }
                big_disks.insert(split[0].to_string(),split[1].to_string());
            }
            Some(big_disks)
        }
        None => None
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

    println!("checking for {:?} in .ssh/config", needed_hosts);
    // check for a config for each server
    for host in &needed_hosts {
        let server_config = ssh_conf.query(host);
        if server_config.is_empty(){
            println!("unable to find a ssh config for {}.\nPlease add a config for {} in {:?} like the following...\nHost {}\n   HostName {}\n   User your_username_on_{}"
            ,host,host,ssh_config,host,host,host);
            return
        }
    }

    // connect to servers
    println!("connecting to hosts ...");
    let mut ssh_connections = HashMap::<String,Host>::new();

    println!("connecting to remote hosts ...");
    for server in &needed_hosts {
        let server_config = ssh_conf.query(server);
        let username = server_config.get("User");
        match username {
            Some(user) => {
                match Host::new(server,user,22) {
                    Err(conn_error) =>{
                        match conn_error {
                            ConnectionError::NoPublicKeysFound => {
                                println!("no ssh public keys found in {:?}\nRun ssh-keygen and make sure you have password-less access to {}",ssh_dir,server);
                                return;
                            }
                            ConnectionError::UnableToConnect => {
                                println!("unable to connect to {}. Make sure you have password-less access!\nYou may need to run ssh-copy-id {}@{}",server,user,server);
                                return
                            }
                            ConnectionError::UnableToStartShell => {
                                println!("unable to start a shell on {}.",server);
                                return
                            }
                            _=> {}
                        }
                    } ,
                    Ok(mut host) => {
                        match host.check_for_server_bin() {
                            Err(_) => {
                                println!("unable to successfully talk to status server on {}.",server);
                                return
                            }
                            Ok(_) => {}
                        }
                        ssh_connections.insert(server.to_string(),host);
                    }
                };
            }
            None => {
                println!("we didn't find a username for {}. Please specify the username in {:?}",server,ssh_config);
                return
            }
        }
    }

    // loop thru stages in pipe to get status
    // if stage is incomplete and a pipe, recurse, append to status report

    //let pipe = conf_col.get_pipe(&args.last_pipeline).unwrap();

    let (status,prog) = conf_col.pipe_status(&args.last_pipeline,&args,&mut ssh_connections,&this_host,&big_disks,&conf_col);

    //let (status,prog) = pipe_status(pipe,args,&mut ssh_connections,&this_host,&big_disks,&conf_col);

    for s in &status{
        let txt = serde_json::to_string_pretty(s).expect("cannot convert to string");
        println!("{}",txt);
    }

    //println!("{:?}",status);

}



// fn pipe_status(pipe:&PipeStatusConfig, args:&ClientArgs,ssh_connections:&mut HashMap<String,Host>, this_host:&String, big_disks:&Option<HashMap<String, String>>,config_collection:&ConfigCollection) -> (Vec<Status>,f32) {
//     println!("running stage checks for {} ...",pipe.label);
//
//     let mut pref_computer = pipe.preferred_computer.clone().unwrap_or(this_host.clone());
//
//     let mut stage_statuses:Vec<Status> = vec![];
//
//     for stage in &pipe.stages {
//
//         println!("building request for {} ...",stage.label);
//         let mut request = Request{
//             stage: stage.clone(),
//             big_disk:None,
//             run_number_list:args.runno_list.clone(),
//             base_runno:args.base_runno.clone(),
//         };
//
//         // overwrite preferred computer if needed
//         match &stage.preferred_computer {
//             Some(computer) => {pref_computer = computer.clone()}
//             None => {}
//         }
//
//         let host = ssh_connections.get_mut(&pref_computer).expect("host not found! what happened??");
//         request.big_disk = match &big_disks {
//             Some(disks) => {
//                 match disks.get(&pref_computer) {
//                     Some(disk) => Some(disk.to_owned()),
//                     None => None
//                 }
//             }
//             None => None
//         };
//         println!("sending request to {}",pref_computer);
//         let stat = match host.submit_request(&request) {
//             Response::Success(status) => status,
//             Response::Error(_) => Status{
//                 label: stage.label.clone(),
//                 progress: StatusType::Invalid,
//                 children: vec![]
//             }
//         };
//
//         println!("status received from {}",pref_computer);
//
//         match &stat.progress {
//             StatusType::NotStarted => { // if a pipe isn't started we have to consider it being a pipe
//
//                 match config_collection.get_pipe(&stage.label) {
//                     Some(pipe) => {
//                         let (children,progress) = pipe_status(pipe,args,ssh_connections,this_host,big_disks,config_collection);
//                         let mut s = if progress == 0.0 {
//                             Status{
//                                 label: stage.label.clone(),
//                                 progress: StatusType::NotStarted,
//                                 children: vec![]
//                             }
//                         }else {
//                             Status{
//                                 label: stage.label.clone(),
//                                 progress: StatusType::InProgress(progress),
//                                 children: vec![]
//                             }
//                         };
//                         s.children = children;
//                         stage_statuses.push(s);
//                     }
//                     None => stage_statuses.push(stat)
//                 }
//             }
//             _=> stage_statuses.push(stat)
//         }
//     }
//
//     // integrate progress
//     let mut total_progress:f32 = 0.0;
//     stage_statuses.iter().for_each(|stat|{
//         match &stat.progress {
//             StatusType::Complete => total_progress = total_progress + 1.0,
//             StatusType::InProgress(progress) => total_progress = total_progress + progress,
//             _=> {}
//         }
//     });
//
//     let frac_progress = total_progress /stage_statuses.len() as f32;
//
//     (stage_statuses,frac_progress)
// }

