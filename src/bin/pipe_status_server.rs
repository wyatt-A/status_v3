use std::path::Path;
use clap::{Parser};
use regex::Regex;
use status_v3::pipe::{ConfigCollection, ConfigCollectionError, SubstitutionTable};
use status_v3::request::{Request, Response, ServerError};
use status_v3::request;
use status_v3::stage::FileCheckError;
use status_v3::status::Status;

#[derive(clap::Parser,Debug)]
struct ServerArgs {
    #[clap(long)]
    request_string:Option<String>,
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

fn process_request(req:&str) -> Result<Status,ServerError> {

    // clean request
    println!("raw request = {:?}",req);

    println!("loading request ...");
    let mut req:Request = serde_json::from_str(req).map_err(|_|ServerError::RequestParse)?;

    println!("parsed request = {:?}",req);

    let big_disk = match &req.big_disk {
        Some(str) => str.to_string(),
        None => std::env::var("BIGGUS_DISKUS").map_err(|_|ServerError::BIGGUS_DISKUS_NotSet)?
    };
    println!("running file check ...");


    let mut table = req.sub_table.to_hash();
    table.insert(String::from("BIGGUS_DISKUS"),big_disk.to_string());
    table.insert(String::from("PARAM0"),req.run_number_list[0].to_string());
    table.insert(String::from("BASE"),req.base_runno.clone().expect("base runno must be defined!"));

    let status = req.stage.file_check(&big_disk,&req.run_number_list,req.base_runno,&table).map_err(|e|ServerError::FileCheckError(e))?;
    Ok(status)
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

    let stage = pipe.stages[0].clone();

    let mut request = Request{
        sub_table: pipe.sub_table(),
        stage: stage.clone(),
        big_disk:None,
        run_number_list:vec!["N60218_m00".to_string()],
        base_runno:Some(String::from("N60218")),
    };

    let stat = process_request(&request.to_json());

    println!("{:?}",stat);

}