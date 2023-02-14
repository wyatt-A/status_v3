// extern crate status_v3;
// use std::path::{Path, PathBuf};
// use clap::builder::TypedValueParser;
// use regex::Regex;
// use status_v3::host;
// use status_v3::pipe::{ConfigCollection, PipeStatusConfig};
// use status_v3::request::Request;
//
// use walkdir::{WalkDir};
// use status_v3::stage::{FileCheckError, FileCounter, Stage};
// use serde::{Serialize,Deserialize};
// use utils::filesystem_search;
//
//
// #[derive(Serialize,Deserialize,Debug,Clone)]
// struct DummyConfig {
//     stages:Vec<Stage>
// }
//
//
// #[test]
// fn test() -> Result<(),FileCheckError>{
//
//
//
//     let cfg = PipeStatusConfig::from_file(Path::new("/Users/Wyatt/IdeaProjects/status_v3/pipe_configs/diffusion_calc_nlsam_status.toml")).unwrap();
//
//     //println!("{:?}",cfg);
//
//     println!("{}",toml::to_string_pretty(&cfg).unwrap());
//
//
//     let big_disk = PathBuf::from(&std::env::var("BIGGUS_DISKUS").expect("cannot get biggus diskus"));
//
//
//     let dummy_stage_file = big_disk.join("dummy_stage.toml");
//     let s = utils::read_to_string(&dummy_stage_file,"toml");
//     let s:Stage = toml::from_str(&s).unwrap();
//
//     //println!("{:?}",s);
//
//     let s = Stage{
//         label: "save_sigma".to_string(),
//         preferred_computer: None,
//         completion_file_pattern: (Regex::new(r"work/.*/stabtest/.*.nii([.]gz)?$").unwrap()),
//         directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-work".to_string(),
//         signature_type: SignatureType::ManyToOne,
//         required_file_keywords: None,
//         file_counter: Some(FileCounter::CountFiles{ multiplier: 0,regex:String::from("dummy")})
//     };
//
//
//     let toml_string = toml::to_string(&s).unwrap();
//     utils::write_to_file(&big_disk.join("save_sigma"),"toml",&toml_string);
//
//
//     // use walkdir to read the stage directory of interest
//     // use regex to filter out the files that we want to consider
//     // somehow determine the number of files we are looking for
//
//
//     // directory resolution
//
//     let dir = big_disk.join("diffusionN60218NLSAMdsi_studio-work");
//
//     // index into pipe config for testing ..
//     //let fc = cfg.stages[7].file_count_resolution.clone();
//
//     let stages = cfg.stages.clone();
//
//
//     for stage in stages {
//         let fc = stage.file_counter.clone();
//         let count = match fc {
//             Some(fc) => {
//                 match fc {
//                     FileCounter::CountFiles {regex,multiplier} => {
//                         let matched_filenames = filesystem_search(&dir, Some(Regex::new(&regex).unwrap()));
//                         matched_filenames.len()*multiplier
//                         //println!("{:?}",matched_filenames);
//                     }
//
//                     FileCounter::FromName{regex} => {
//                         let re = Regex::new(&regex).unwrap();
//                         let matched_files = filesystem_search(&dir,Some(re.clone()));
//                         if matched_files.is_empty() {
//                             return Err(FileCheckError::RequiredFileNotFound)
//                         }
//                         // if matched_files.len() > 1 {
//                         //     return Err(FileCheckError::PatternTooGenerous)
//                         // }
//                         let caps = re.captures(&matched_files[0]).unwrap();
//                         let capture = caps.get(1).unwrap().as_str();
//                         capture.parse().unwrap()
//                     }
//                     FileCounter::Constant {count} => count,
//                     FileCounter::FromNameDerived {regex,dep_regex,dep_multiplier} => {
//
//                         let re = Regex::new(&regex).unwrap();
//                         let matched_files = filesystem_search(&dir,Some(re.clone()));
//                         if matched_files.is_empty() {
//                             return Err(FileCheckError::RequiredFileNotFound)
//                         }
//                         let caps = re.captures(&matched_files[0]).unwrap();
//                         let capture = caps.get(1).unwrap().as_str();
//                         let from_name_int:usize = capture.parse().unwrap();
//                         let matched_filenames = filesystem_search(&dir, Some(Regex::new(&dep_regex).unwrap()));
//                         let multiplier = matched_filenames.len()*dep_multiplier;
//                         multiplier*from_name_int
//
//                     }
//                     _=> {
//                         1
//                     }
//                 }
//             }
//             None => {
//                 1
//             }
//         };
//     println!("stage:{},count = {}",stage.label,count);
//     }
//
//
//
//     // //todo!(check that strings are valid regex on load,not here in the "server code")
//     // let required_count = match fc {
//     //     Some(fc) => {
//     //         match fc {
//     //             FileCountResolution::CountFiles {regex,multiplier} => {
//     //                 let matched_filenames = filesystem_search(&dir, Some(Regex::new(&regex).unwrap()));
//     //                 matched_filenames.len()*multiplier
//     //                 //println!("{:?}",matched_filenames);
//     //             }
//     //             FileCountResolution::FromName{regex} => {
//     //                 let re = Regex::new(&regex).unwrap();
//     //                 let matched_files = filesystem_search(&dir,Some(re.clone()));
//     //                 if matched_files.is_empty() {
//     //                     return Err(FileCheckError::RequiredFileNotFound)
//     //                 }
//     //                 // if matched_files.len() > 1 {
//     //                 //     return Err(FileCheckError::PatternTooGenerous)
//     //                 // }
//     //                 let caps = re.captures(&matched_files[0]).unwrap();
//     //                 let capture = caps.get(1).unwrap().as_str();
//     //                 capture.parse().unwrap()
//     //             }
//     //             FileCountResolution::Constant {count} => count,
//     //             _=> {
//     //                 1
//     //             }
//     //         }
//     //     }
//     //     None => {
//     //         1
//     //     }
//     // };
//
//     // println!("required count = {}",required_count);
//
//     Ok(())
// }
//
//
