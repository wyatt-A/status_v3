extern crate status_v3;
use std::path::Path;
use status_v3::host;
use status_v3::pipe::ConfigCollection;
use status_v3::request::Request;

#[test]
fn test(){
    println!("tree testing");

    let cfg = ConfigCollection::from_dir(Path::new("./pipe_configs"));

    let pipe_name = "diffusion_calc";

    let pipe = cfg.get_pipe(pipe_name).expect("pipe not found");

    println!("{:?}",pipe);

    // build stage list

    // first breadth-first search for stages that are also pipes
    let child_pipes = vec![];



    // pipe.stages.iter().flat_map(|stage|{
    //     match cfg.get_pipe(&stage.label) {
    //         Some(pipe) => pipe
    //     }
    // } )
    //
    // for stage in &pipe.stages {
    //
    // }
}