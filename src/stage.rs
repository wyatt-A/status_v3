use std::collections::HashMap;
use std::path::{Path};
use regex::Regex;
use serde::{Serialize,Deserialize};
use crate::status::{Status, StatusType};
use utils;
use crate::pipe::SubstitutionTable;

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct Stage {
    pub label:String,
    pub preferred_computer:Option<String>,
    pub completion_file_pattern:String,
    pub directory_pattern:String,
    pub required_file_keywords:Option<Vec<String>>,
    pub file_counter:Option<FileCounter>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub enum FileCheckError {
    InvalidRegex,
    SignatureTypeNotImplemented,
    RequiredFileKeywordsNotFound,
    BaseRunNumberMustBeSpecified,
    PatternTooGenerous,
    RequiredFileNotFound,
    RegexCaptureNotFound,
    IntParseError,
    ExpectedRegexMatch,
    NoExpectedMatches,
    SubstitutionNotResolved(String),
}


impl Stage {

    pub fn file_check(&self, _big_disk:&String, _runno_list:&Vec<String>, _base_runno:&String,sub_table:&HashMap<String,String>) -> Result<Status,FileCheckError> {

        let big_disk_resolved = substitute(&self.directory_pattern,sub_table)?;
        let file_completion_pattern = substitute(&self.completion_file_pattern,sub_table)?;


        //println!("big disk resolved = {}",big_disk_resolved);

        let matched_files = utils::filesystem_search(Path::new(&big_disk_resolved),Some(Regex::new(&file_completion_pattern).map_err(|_|FileCheckError::InvalidRegex)?));

        // for file in &matched_files {
        //     println!("{}",file);
        // }

        let n_matched_files = matched_files.len();


        let n_expected_matches = match &self.file_counter {
            Some(counter) => {
                counter.count(Path::new(&big_disk_resolved))?
            }
            None => 1
        };

        if n_expected_matches == 0 {
            return Err(FileCheckError::NoExpectedMatches)
        }

        if n_matched_files == 0 {
            return Ok(Status{
                label: self.label.to_string(),
                progress: StatusType::NotStarted,
                children: vec![]
            })
        }

        if n_matched_files == n_expected_matches {
            return Ok(Status{
                label: self.label.to_string(),
                progress: StatusType::Complete,
                children: vec![]
            })
        }

        let progress = n_matched_files as f32/n_expected_matches as f32;
        Ok(Status{
            label: self.label.to_string(),
            progress: StatusType::InProgress(progress),
            children: vec![]
        })
    }
}

fn substitute(thing_to_resolve:&String,sub_table:&HashMap<String,String>) -> Result<String,FileCheckError> {
    let re = Regex::new(r"(\$\{[[:alnum:]_]+\})").map_err(|_|FileCheckError::InvalidRegex)?;
    let mut output = thing_to_resolve.clone();
    for captures in re.captures_iter(&thing_to_resolve) {
        for cap_idx in 1..captures.len(){
            let cap:String = captures[cap_idx].to_string();
            // we include the ${ } in capture because we want to replace it, but it is not in the hash
            let subtr = &cap[2..cap.len()-1];
            let rep = match sub_table.get(subtr) {
                Some(sub) => {
                    sub.to_string()
                },
                None => Err(FileCheckError::SubstitutionNotResolved(subtr.to_string()))?
            };
            output = output.replace(&format!("{}",&captures[cap_idx]),&rep);
        }
    }
    Ok(output)
}

// how to determine the number of files we are matching to
#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(tag = "type")]
pub enum FileCounter {
    ListFile,
    Constant{count:usize},
    FromName{regex:String},
    FromNameDerived{regex:String,dep_regex:String,dep_multiplier:usize},
    CountFiles{regex:String,multiplier:usize},
}

impl FileCounter {
    pub fn count(&self,dir:&Path) -> Result<usize,FileCheckError> {
        use FileCounter::*;
        let count = match &self {
            CountFiles {regex,multiplier} => {
                let matched_filenames = utils::filesystem_search(&dir, Some(Regex::new(&regex).map_err(|_|FileCheckError::InvalidRegex)?));
                matched_filenames.len()*multiplier
            }
            FromName{regex} => {
                let re = Regex::new(&regex).map_err(|_|FileCheckError::InvalidRegex)?;
                let matched_files = utils::filesystem_search(&dir,Some(re.clone()));
                if matched_files.is_empty() {
                    Err(FileCheckError::RequiredFileNotFound)?
                }
                let caps = re.captures(&matched_files[0]).ok_or(FileCheckError::RequiredFileNotFound)?;
                let capture = caps.get(1).ok_or(FileCheckError::RegexCaptureNotFound)?.as_str();
                capture.parse().map_err(|_|FileCheckError::IntParseError)?
            }
            Constant {count} => *count,
            FromNameDerived {regex,dep_regex,dep_multiplier} => {
                let re = Regex::new(&regex).map_err(|_|FileCheckError::InvalidRegex)?;
                let matched_files = utils::filesystem_search(&dir,Some(re.clone()));
                if matched_files.is_empty() {
                    Err(FileCheckError::RequiredFileNotFound)?
                }
                let caps = re.captures(&matched_files[0]).ok_or(FileCheckError::ExpectedRegexMatch)?;
                let capture = caps.get(1).ok_or(FileCheckError::RegexCaptureNotFound)?.as_str();
                let from_name_int:usize = capture.parse().map_err(|_|FileCheckError::IntParseError)?;
                let matched_filenames = utils::filesystem_search(&dir, Some(Regex::new(&dep_regex).map_err(|_|FileCheckError::InvalidRegex)?));
                let multiplier = matched_filenames.len()*dep_multiplier;
                multiplier*from_name_int
            }
            _=> 1
        };
        Ok(count)
    }
}
