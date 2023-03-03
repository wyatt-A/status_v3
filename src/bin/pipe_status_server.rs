use clap::{Parser};
use status_v3::pipe::{ConfigCollection, ConfigCollectionError, SubstitutionTable};
use status_v3::request::{Request, Response, ServerError};
use status_v3::server::process_request;

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