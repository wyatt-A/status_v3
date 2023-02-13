use std::path::{Path, PathBuf};
use regex::Regex;
use serde::{Serialize,Deserialize};
use crate::status::{Status, StatusType};



#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Stage {
    pub label:String,
    pub preferred_computer:Option<String>,
    #[serde(with = "serde_regex")]
    pub completion_file_pattern:Regex,
    pub directory_pattern:String,
    pub signature_type:SignatureType,
    pub required_file_keywords:Option<Vec<String>>,
    //pub resolver:Resolver
    //pub count_resolution:Option<FileCountResolution>,
    //pub count_multiplier:Option<f32>,
    //#[serde(with = "serde_regex")]
    //pub count_name_parse:Option<Regex>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum FileCheckError {
    InvalidRegex,
    SignatureTypeNotImplemented,
    RequiredFileKeywordsNotFound,
    BaseRunNumberMustBeSpecified,
}


impl Stage {
    pub fn file_check(&self, big_disk:&str, runno_list:&Vec<String>, base_runno:Option<String>) -> Result<Status,FileCheckError> {
        use SignatureType::*;

        //println!("stage label: {}",self.label);

        let re = Regex::new(r"(\$\{[[:alnum:]_]+\})").map_err(|_|FileCheckError::InvalidRegex)?;

        // may have user arg BIGGUS_DISKUS
        // it an optional hostname : alternate_biggus
        // when it has no hostname, OR hostname matches this host, we use it.

        let mut the_dir = self.directory_pattern.clone();

        println!("the dir = {:?}",the_dir);

        re.captures_iter(&self.directory_pattern).for_each(|captures|{
            for cap_idx in 1..captures.len(){
                if captures[cap_idx].eq("${BIGGUS_DISKUS}") {
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&big_disk);
                }else if captures[cap_idx].eq("${PARAM0}") {
                    //todo(deal with 0 match?)
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&runno_list[0]);
                }else if captures[cap_idx].eq("${PREFIX}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"diffusion");
                }
                else if captures[cap_idx].eq("${SUFFIX}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"");
                }
                else if captures[cap_idx].eq("${PROGRAM}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"dsi_studio");
                }
                else if captures[cap_idx].eq("${BASE}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),&base_runno.clone().unwrap_or("BASE_RUNNO".to_string()));
                }
                else if captures[cap_idx].eq("${SEP}"){
                    the_dir = the_dir.replace(&format!("{}",&captures[cap_idx]),"");
                }else {
                    panic!("capture not recognized")
                }
            }
        });

        // directory should be valid now... if its missing we cant check?... or this stage has 0 progress
        // return not started?

        println!("runno list = {:?}",runno_list);
        println!("\tresolved directory pattern :{:?}",the_dir);



        // let n = match &self.count_resolution {
        //     Some(method) => {
        //         match method {
        //             FileCountResolution::CountFiles(re) => {
        //                 // use regex to get count from matches
        //                 1
        //             }
        //             FileCountResolution::ListFile(pattern) => {
        //                 // use the pattern to open a list file to extract n
        //                 1
        //             }
        //             FileCountResolution::Anything => 1,
        //             FileCountResolution::Match(pattern) => 1
        //         }
        //     }
        //     None => 1
        // };
        //
        // let required_matches = match &self.signature_type {
        //
        //     ToK => {
        //         vec![]
        //     }
        //
        //
        //     _=> Err(FileCheckError::SignatureTypeNotImplemented)?
        //
        // };


        // trim required matches based on signature type=
        // this was too clever for our "better" signature types.
        // we stuffed the meaning of K into the lenght of required matches, making this not extesnsible.

        let required_matches = match &self.signature_type {
            ManyToOne => {
                // we will assume only the base runno is involved in the match
                vec![base_runno.ok_or(FileCheckError::BaseRunNumberMustBeSpecified)?.to_string()]
            }
            ManyToMany => {
                runno_list.clone()
            }

            OneToMany => {
                //self.required_file_keywords.clone().expect("you need to specify required file keywords for OneToMany signature pattern")
                self.required_file_keywords.clone().ok_or(FileCheckError::RequiredFileKeywordsNotFound)?
            }
            OneToOne => {
                // relies on completion file pattern to filter to the ONLY thing which should match.
                vec![".".to_string()]
            }
            _=> {
                Err(FileCheckError::SignatureTypeNotImplemented)?
            }
        };

        let contents = match std::fs::read_dir(&the_dir) {
            Err(_) => return Ok(Status{
                label: self.label.clone(),
                progress: StatusType::NotStarted,
                children: vec![]
            }),
            Ok(contents) => contents
        };

        let mut included = vec![];
        for thing in contents {
            let tp = thing.unwrap();
            let file_name = tp.path().file_name().unwrap().to_string_lossy().into_owned();
            let file_path = tp.path().to_string_lossy().into_owned();
            if self.completion_file_pattern.is_match(&file_path) {
                included.push(file_name)
            }
        }

        println!("\t\tconsidered items: {:?}",included);
        println!("\t\tpattern: {:?}",self.completion_file_pattern);

        included.sort();
        let glob = included.join("/");

        let mut count = 0;

        for rm in &required_matches {
            if Regex::new(&format!("(^|/).*{}.*($|/)",rm)).unwrap().is_match(&glob){
                count = count + 1;
                //println!("{}",rm);
            }
        }

        return if count == required_matches.len() {
            Ok(Status{
                label: self.label.clone(),
                progress: StatusType::Complete,
                children: vec![]
            })
        } else if count == 0 {
            Ok(Status{
                label: self.label.clone(),
                progress: StatusType::NotStarted,
                children: vec![]
            })
        } else {
            Ok(Status{
                label: self.label.clone(),
                progress: StatusType::InProgress(count as f32 / required_matches.len() as f32),
                children: vec![]
            })
        }
    }
}






// how to determine the patterns we will match for stage completion
#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum SignatureType {
    // ToN,
    // ToKN,
    // ToK,
    // ToZ,
    ManyToOne,
    ManyToMany,
    OneToMany,
    OneToOne,
}

// how to determine the number of files we are matching to
// #[derive(Serialize,Deserialize,Debug,Clone)]
// pub enum FileCountResolution {
//     // use this regex to return the number of files we are expecting
//     #[serde(with = "serde_regex")]
//     CountFiles(Regex),
//     //somehow this resolves to a path to a list file
//     // on load, it uses the volume runnos as required matches (incompatible with required file keywords)
//     ListFile(String),
//     #[serde(with = "serde_regex")]
//     CountFileInName(Regex),
//     Constant(usize),
// }

// impl FileCountResolution {
//     pub fn len(&self) -> usize {
//         use FileCountResolution::*;
//         match &self {
//             CountFiles(re) => 1,
//             CountFileInName(re) => 1,
//             Constant(num) => *num,
//             ListFile(filepath) => 1,
//         }
//     }
// }
//
//
//
// #[derive(Serialize,Deserialize,Debug,Clone)]
// struct Resolver {
//     counter:FileCountResolution,
//     #[serde(with = "serde_regex")]
//     pattern:Regex
// }
//
//
//
//
//
// // determines the required patterns to match the will determine the stage progress
// #[derive(Serialize,Deserialize,Debug,Clone)]
// pub enum PatternResolution {
//     RunNumberList(FileCountResolution),
//     BaseRunNumber(FileCountResolution),
//
// }



/*
Given a stage, determine


// given a count resolution, return the number of things that must be found (files)
n_expected = CountResolution.len()


// build a list of regex patterns to search for, this may need a count
n_found = CountPatterns.find()

(n_found/n_expected) -> Status

 */

