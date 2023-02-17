use std::path::Path;
use crate::request::{Request, ServerError};
use crate::status::Status;

pub fn process_request(req:&str) -> Result<Status,ServerError> {

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


    let mut runny_list = match req.run_number_list.is_empty() {
        true => {
            let list_file = Path::new(&big_disk).join(&req.base_runno).with_extension("list");
            let runno_list:Vec<String> = match list_file.exists(){
                true => {
                    println!("LIST FILE FOUND!");
                    let s = utils::read_to_string(&list_file,Some("list"));
                    let re = regex::Regex::new(r",|\s+").unwrap();
                    re.split(s.as_str()).map(|s| s.to_string()).collect()
                }
                false => {
                    vec![]
                }
            };
            runno_list
        }
        false => {
            req.run_number_list.clone()
        }
    };

    let mut table = req.sub_table.to_hash();
    table.insert(String::from("BIGGUS_DISKUS"),big_disk.to_string());
    if !runny_list.is_empty(){
        table.insert(String::from("PARAM0"),runny_list[0].to_string());
    }
    table.insert(String::from("BASE"),req.base_runno.clone());


    let stat = match req.stage.label.as_str() {
        "archive" => {
            req.stage.archive_check(&table).map_err(|e|ServerError::FileCheckError(e))?
        }
        _=> { // run file check for normal stage
            req.stage.file_check(&big_disk, &runny_list, &req.base_runno, &table).map_err(|e|ServerError::FileCheckError(e))?
        }
    };

    Ok(stat)
}

// pub fn process_archive_request(req:&str) -> Result<Status,ServerError> {
//     let mut req:ArchiveCheckRequest = serde_json::from_str(req).map_err(|_|ServerError::RequestParse)?;
//
//     println!("parsed request = {:?}",req);
//
//     let big_disk = match &req.big_disk {
//         Some(str) => str.to_string(),
//         None => std::env::var("BIGGUS_DISKUS").map_err(|_|ServerError::BIGGUS_DISKUS_NotSet)?
//     };
//
//     let mut runny_list = match req.run_number_list.is_empty() {
//         true => {
//             let list_file = Path::new(&big_disk).join(&req.base_runno).with_extension("list");
//             let runno_list:Vec<String> = match list_file.exists(){
//                 true => {
//                     println!("LIST FILE FOUND!");
//                     let s = utils::read_to_string(&list_file,Some("list"));
//                     let re = regex::Regex::new(r",|\s+").unwrap();
//                     re.split(s.as_str()).map(|s| s.to_string()).collect()
//                 }
//                 false => {
//                     vec![]
//                 }
//             };
//             runno_list
//         }
//         false => {
//             req.run_number_list.clone()
//         }
//     };
//
//     let mut table = req.sub_table.to_hash();
//     table.insert(String::from("BIGGUS_DISKUS"),big_disk.to_string());
//     if !runny_list.is_empty(){
//         table.insert(String::from("PARAM0"),runny_list[0].to_string());
//     }
//     table.insert(String::from("BASE"),req.base_runno.clone());
//
//     req.archive_check.archive_check(&table)
//
//
//
// }