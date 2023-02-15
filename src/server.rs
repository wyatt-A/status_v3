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


    let mut table = req.sub_table.to_hash();
    table.insert(String::from("BIGGUS_DISKUS"),big_disk.to_string());
    table.insert(String::from("PARAM0"),req.run_number_list[0].to_string());
    table.insert(String::from("BASE"),req.base_runno.clone());

    let status = req.stage.file_check(&big_disk,&req.run_number_list,&req.base_runno,&table).map_err(|e|ServerError::FileCheckError(e))?;
    Ok(status)

}