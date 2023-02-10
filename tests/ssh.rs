extern crate status_v3;

use std::path::Path;
use status_v3::host;
use status_v3::pipe::ConfigCollection;
use status_v3::request::Request;


#[test]
fn test(){
    println!("this is a test");

    let mut h = host::Host::new("seba","wyatt",22).expect("unable to connect");

    let config_collection = ConfigCollection::from_dir(Path::new("./pipe_configs"));

    let p = config_collection.get_pipe("co_reg").unwrap();

    let s = p.stages[0].clone();

    let req = Request{
        stage: s,
        big_disk: None,
        run_number_list: vec![]
    };

    println!("submitting request");
    let response = h.submit_request(&req);

    println!("{:?}",response);
}