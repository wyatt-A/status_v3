use clap::{Parser};
use status_v3::request::{Request, Response, ServerError};
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

    fn process_request(req:&str) -> Result<Status,ServerError> {

        // clean request
        println!("raw request = {:?}",req);

        println!("loading request ...");
        let req:Request = serde_json::from_str(req).map_err(|_|ServerError::RequestParse)?;

        println!("parsed request = {:?}",req);

        let big_disk = match &req.big_disk {
            Some(str) => str.to_string(),
            None => std::env::var("BIGGUS_DISKUS").map_err(|_|ServerError::BIGGUS_DISKUS_NotSet)?
        };
        println!("running file check ...");

        let status = req.stage.file_check(&big_disk,&req.run_number_list,req.base_runno).map_err(|e|ServerError::FileCheckError(e))?;
        Ok(status)
    }
}