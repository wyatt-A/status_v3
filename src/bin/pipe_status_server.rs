use std::path::Path;
use clap::{Parser};
use regex::Regex;
use status_v3::pipe::{ConfigCollection, ConfigCollectionError, SubstitutionTable};
use status_v3::request::{Request, Response, ServerError};
use status_v3::request;
use status_v3::server::process_request;
use status_v3::stage::FileCheckError;
use status_v3::status::Status;

#[derive(clap::Parser,Debug)]
struct ServerArgs {
    #[clap(long)]
    request_string:Option<String>,
    /// Turn debugging information on
    #[clap(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

fn main(){
    let args:ServerArgs = ServerArgs::parse();
    match &args.request_string {
        Some(request) => run_server(request),
        None => {
            println!("pipe_status_server:=hi there ;)");
        }
    }
}

fn run_server(request_json:&str){

    let re = match process_request(request_json) {
        Err(e) => Response::Error(e),
        Ok(stat) => Response::Success(stat)
    };
    let resp_string = serde_json::to_string(&re).expect("unable to serialize response");
    print!("||{}||",resp_string);
}


#[test]
fn server_test(){

    let pipe_conf_dir = Path::new("/Users/Wyatt/IdeaProjects/status_v3/pipe_configs");


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

    let pipe = conf_col.get_pipe("diffusion_calc_nlsam").unwrap();

    let stage = pipe.stages[11].clone();

    let mut request = Request{
        sub_table: pipe.sub_table(),
        stage: stage.clone(),
        big_disk:None,
        run_number_list:vec!["N60218_m00".to_string()],
        base_runno:String::from("N60218"),
    };

    let stat = process_request(&request.to_json());

    println!("{:?}",stat);

}
