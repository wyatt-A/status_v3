extern crate status_v3;
use std::path::{Path, PathBuf};
use regex::Regex;
use status_v3::host;
use status_v3::pipe::ConfigCollection;
use status_v3::request::Request;

use walkdir::{WalkDir,Error};

#[test]
fn test() -> Result<(),Error>{
    // use walkdir to read the stage directory of interest
    // use regex to filter out the files that we want to consider
    // somehow determine the number of files we are looking for
    let big_disk = PathBuf::from(&std::env::var("BIGGUS_DISKUS").expect("cannot get biggus diskus"));

    // directory resolution

    let dir = big_disk.join("diffusionN60218NLSAMdsi_studio-work");

    // recursive directory read

    let mut files = vec![];
    for entry in WalkDir::new(dir) {
        //println!("{}", entry?.path().display());
        files.push(entry?.path().to_string_lossy().to_string());
    }

    //println!("{:?}",files);


    let re = Regex::new(r"work/.*/stabtest/.*.nii([.]gz)?$").expect("invalid regex");


    let matches:Vec<String> = files.iter().flat_map(|thing|{
        match re.is_match(thing) {
            true => Some(thing.to_string()),
            false => None
        }
    }).collect();


    // let matches:Vec<&str> = files.iter().filter(|filepath|{
    //     re.is_match(filepath)
    // }).collect();

    println!("{:?}",matches);

    Ok(())


}