extern crate status_v3;
use std::path::Path;
use status_v3::host;
use status_v3::pipe::ConfigCollection;
use status_v3::request::Request;


#[test]
fn test(){
    println!("{:?}",whoami::username());
}