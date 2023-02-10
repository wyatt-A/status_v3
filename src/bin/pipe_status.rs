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
use status_v3::host::Host;
use status_v3::pipe::{ConfigCollection, PipeStatusConfig};
use status_v3::request::{Request, Response, ServerError};
use status_v3::status::{Status, StatusType};
use whoami;

#[derive(clap::Parser,Debug)]
struct ClientArgs {
    last_pipeline:String,
    runno_list:Vec<String>,
    #[clap(short, long)]
    big_disk:Option<Vec<String>>,
    #[clap(short, long)]
    pipe_configs:Option<PathBuf>,
    #[clap(short, long)]
    run_server:Option<String>
}

fn main(){

    let args:ClientArgs = ClientArgs::parse();
    run_client(&args);
}

fn run_client(args:&ClientArgs){

    let this_exe = std::env::current_exe().expect("cannot determine this program!");

    let home_dir:PathBuf = dirs::home_dir().expect("cannot get home directory!");
    let this_host = utils::computer_name();
    let this_user = whoami::username();

    println!("loading config files ...");
    // load pipe configs
    //let pipe_config_dir = args.pipe_configs.clone().unwrap_or(home_dir.join(".pipe_configs"));
    let pipe_config_dir = args.pipe_configs.clone().unwrap_or(PathBuf::from("./pipe_configs"));

    let conf_col = ConfigCollection::from_dir(&pipe_config_dir);

    println!("resolving pipeline hosts ...");
    // get list of hosts from last_pipe
    let needed_servers = conf_col.servers(&args.last_pipeline);

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
    let ssh_config_file = home_dir.join(".ssh").join("config");
    if !ssh_config_file.exists(){
        println!("no ssh config found! You need to set this up first!");
        return
    }

    // load ssh config file and parse it to a usable type
    let config_str = utils::read_to_string(&ssh_config_file,"");
    let c = SSHConfig::parse_str(&config_str).unwrap();


    println!("checking for {:?} in .ssh/config",needed_servers);
    // check for a config for each server
    for server in &needed_servers {
        let server_config = c.query(server);
        if server_config.is_empty(){
            println!("we didn't find a ssh config for {} in your .ssh/config file. Please add the host the the file!",server);
            return
        }
    }

    // connect to servers
    println!("connecting to hosts ...");
    let mut ssh_connections = HashMap::<String,Host>::new();

    // connect to localhost
    // println!("connecting to localhost ...");
    // let mut localhost = Host::new(&this_host,&this_user,22).expect(&format!("unable to connect to localhost {}@{}",this_user,this_host));
    // localhost.check_for_bin();
    // ssh_connections.insert(this_host.clone(),localhost);

    println!("connecting to remote hosts ...");
    for server in &needed_servers {
        let server_config = c.query(server);
        let username = server_config.get("User");
        match username {
            Some(user) => {
                match Host::new(server,user,22) {
                    Err(_) => println!("unable to connect to {}. Make sure you have password-less access! You may need to run ssh-copy-id {}@{}",server,user,server),
                    Ok(mut host) => {
                        host.check_for_bin();
                        ssh_connections.insert(server.to_string(),host);
                    }
                };
            }
            None => {
                println!("we didn't find a username for {}. Please specify the username in .ssh/config",server);
                return
            }
        }
    }


    // protect connections with an reference-counted mutable ref-cell
    let conn = Rc::new(RefCell::new(ssh_connections));


    // loop thru stages in pipe to get status
    // if stage is incomplete and a pipe, recurse, append to status report

    let pipe = conf_col.get_pipe(&args.last_pipeline).unwrap();
    let status = pipe_status(pipe,args,conn,&this_host,&big_disks,&conf_col);
    println!("{:?}",status);
}



fn pipe_status(pipe:&PipeStatusConfig, args:&ClientArgs,ssh_connections:Rc<RefCell<HashMap<String,Host>>>, this_host:&String, big_disks:&Option<HashMap<String, String>>,config_collection:&ConfigCollection) -> (Vec<Status>,f32) {
    println!("running stage checks for {} ...",pipe.label);

    let ssh_conn = ssh_connections.clone();
    let mut ssh_conn = ssh_conn.as_ref().borrow_mut();

    //let mut preferred_computers = pipe.preferred_computer.clone().unwrap_or(vec![]);
    let mut preferred_computers = pipe.preferred_computer.clone().unwrap_or(vec![this_host.clone()]);

    let mut pref_computer = &preferred_computers[0];

    let mut stage_statuses:Vec<Status> = vec![];

    for stage in &pipe.stages {

        println!("building request for {} ...",stage.label);
        let mut request = Request{
            stage: stage.clone(),
            big_disk:None,
            run_number_list:args.runno_list.clone(),
        };

        // append to preferred computers
        match &stage.preferred_computer {
            Some(computers) => {
                // for computer in computers {
                //     preferred_computers.insert(0,computer.clone());
                // }
                pref_computer = &computers[0];
            }
            None => {}
        }

        let host = ssh_conn.get_mut(pref_computer).expect("host not found! what happened??");
        request.big_disk = match &big_disks {
            Some(disks) => {
                match disks.get(pref_computer) {
                    Some(disk) => Some(disk.to_owned()),
                    None => None
                }
            }
            None => None
        };
        println!("sending request to {}",pref_computer);
        let stat = match host.submit_request(&request) {
            Response::Success(status) => status,
            Response::Error(_) => Status{
                label: stage.label.clone(),
                progress: StatusType::Invalid,
                children: vec![]
            }
        };

        println!("returned status = {:?}",stat);

        match &stat.progress {
            StatusType::NotStarted => { // if a pipe isn't started we have to consider it being a pipe

                match config_collection.get_pipe(&stage.label) {
                    Some(pipe) => {
                        let (children,progress) = pipe_status(pipe,args,ssh_connections.clone(),this_host,big_disks,config_collection);
                        let mut s = if progress == 0.0 {
                            Status{
                                label: stage.label.clone(),
                                progress: StatusType::NotStarted,
                                children: vec![]
                            }
                        }else {
                            Status{
                                label: stage.label.clone(),
                                progress: StatusType::InProgress(progress),
                                children: vec![]
                            }
                        };
                        s.children = children;
                        stage_statuses.push(s);
                    }
                    None => stage_statuses.push(stat)
                }
            }
            _=> stage_statuses.push(stat)
        }
    }

    // integrate progress
    let mut total_progress:f32 = 0.0;
    stage_statuses.iter().for_each(|stat|{
        match &stat.progress {
            StatusType::Complete => total_progress = total_progress + 1.0,
            StatusType::InProgress(progress) => total_progress = total_progress + progress,
            _=> {}
        }
    });

    let frac_progress = total_progress /stage_statuses.len() as f32;

    (stage_statuses,frac_progress)
}












