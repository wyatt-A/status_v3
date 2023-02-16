use std::collections::HashMap;
use std::path::{Path};
use std::process::Command;
use regex::Regex;
use serde::{Serialize,Deserialize};
use crate::status::{Status, StatusType};
use utils;
use crate::host::{DbResponse, DBStatus};
use crate::pipe::{substitute, SubstitutionTable};

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
pub struct ArchiveCheck {
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




impl ArchiveCheck {
    pub fn archive_check(&self,sub_table:&HashMap<String,String>) -> Result<bool,FileCheckError> {
        let big_disk_resolved = substitute(&self.directory_pattern,sub_table)?;
        let file_completion_pattern = substitute(&self.completion_file_pattern,sub_table)?;

        let n_expected_archived = match &self.file_counter {
            Some(counter) => {
                counter.count(Path::new(&big_disk_resolved),sub_table)?
            }
            None => 1
        };

        let result = civm_db_exists(&vec![file_completion_pattern]);

        let mut n_found = 0;
        for (_,stat) in result {
            match stat {
                DBStatus::Found => {
                    n_found = n_found + 1;
                }
                _=> {}
            }
        }
        Ok(n_found == n_expected_archived)
    }
}

pub fn civm_db_exists(things:&Vec<String>) -> HashMap<String,DBStatus> {
    let cmd = "civm_db_check";
    let args:Vec<String> = things.iter().map(|thing|format!("--exists={}",thing)).collect();
    let re = Regex::new(r#"(\{"exists":.*?\}\})"#).expect("invalid regex");
    println!("running db check locally ...");
    let o = Command::new(cmd).args(&args).output().expect("failed to launch db check");
    let r = String::from_utf8(o.stdout.clone()).unwrap();
    let cap = re.captures(&r).expect("command response not matched").get(1).expect("command response").as_str().to_string();
    let dbr:DbResponse = serde_json::from_str(&cap).expect("unable to deserialize database response");
    let h = dbr.to_hash();
    println!("{:?}",h);
    h
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
                counter.count(Path::new(&big_disk_resolved),sub_table)?
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



// how to determine the number of files we are matching to
#[derive(Serialize,Deserialize,Debug,Clone)]
#[serde(tag = "type")]
pub enum FileCounter {
    ListFile,
    Constant{count:usize},
    FromName{regex:String},
    FromContentDerived{file_pattern:String,regex:String,dep_regex:String,dep_multiplier:usize},
    FromNameDerived{regex:String,dep_regex:String,dep_multiplier:usize,use_sum:Option<bool>},
    CountFiles{regex:String,multiplier:usize},
}

impl FileCounter {
    pub fn count(&self,dir:&Path,sub_table:&HashMap<String,String>) -> Result<usize,FileCheckError> {
        use FileCounter::*;
        let count = match &self {
            CountFiles {regex,multiplier} => {
                let regex = substitute(&regex,sub_table)?;
                let matched_filenames = utils::filesystem_search(&dir, Some(Regex::new(&regex).map_err(|_|FileCheckError::InvalidRegex)?));
                matched_filenames.len()*multiplier
            }
            FromName{regex} => {
                let regex = substitute(&regex,sub_table)?;
                let re = Regex::new(&regex).map_err(|_|FileCheckError::InvalidRegex)?;
                let matched_files = utils::filesystem_search(&dir,Some(re.clone()));
                if matched_files.is_empty() {
                    Err(FileCheckError::RequiredFileNotFound)?
                }
                let caps = re.captures(&matched_files[0]).ok_or(FileCheckError::RequiredFileNotFound)?;
                let capture = caps.get(1).ok_or(FileCheckError::RegexCaptureNotFound)?.as_str();
                capture.parse().map_err(|_|FileCheckError::IntParseError)?
            }
            FromContentDerived{file_pattern,regex,dep_regex,dep_multiplier} => {
                let regex = &substitute(&regex,sub_table)?;
                let dep_regex = &substitute(&dep_regex,sub_table)?;
                let re = Regex::new(&regex).map_err(|_|FileCheckError::InvalidRegex)?;
                let matched_files = utils::filesystem_search(&dir,Some(re.clone()));
                if matched_files.is_empty() {
                    Err(FileCheckError::RequiredFileNotFound)?
                }
                // what about the complicated case where we should get the number from each mached file?
                // we should still have the proper number of matched files, but instead of getting number from the first and multipling,
                // we need to sum the number from each.
                let dep_filenames = utils::filesystem_search(&dir, Some(Regex::new(&dep_regex).map_err(|_|FileCheckError::InvalidRegex)?));
                let expected_file_count = dep_filenames.len()*dep_multiplier;

                // open file from first match
                let file_pattern = Regex::new(file_pattern).map_err(|_|FileCheckError::InvalidRegex)?;
                let count_re = Regex::new(regex).map_err(|_|FileCheckError::InvalidRegex)?;
                let matched_files = utils::filesystem_search(&dir,Some(file_pattern.clone()));
                let content_integer:usize = match matched_files.len(){
                    0 => Err(FileCheckError::RequiredFileNotFound)?,
                    _=> {
                        let file_contents = utils::read_to_string(Path::new(&matched_files[0]),None);
                        let caps = count_re.captures(&file_contents).ok_or(FileCheckError::ExpectedRegexMatch)?;
                        let cap_string = caps.get(1).ok_or(FileCheckError::RegexCaptureNotFound)?;
                        cap_string.as_str().parse().map_err(|_|FileCheckError::IntParseError)?
                    }
                };
                expected_file_count*content_integer
            }
            Constant {count} => *count,
            FromNameDerived {regex,dep_regex,dep_multiplier,use_sum} => {
                let regex = &substitute(&regex,sub_table)?;
                let dep_regex = &substitute(&dep_regex,sub_table)?;
                let re = Regex::new(&regex).map_err(|_|FileCheckError::InvalidRegex)?;
                let matched_files = utils::filesystem_search(&dir,Some(re.clone()));
                if matched_files.is_empty() {
                    Err(FileCheckError::RequiredFileNotFound)?
                }
                // what about the complicated case where we should get the number from each mached file?
                // we should still have the proper number of matched files, but instead of getting number from the first and multipling,
                // we need to sum the number from each.
                let dep_filenames = utils::filesystem_search(&dir, Some(Regex::new(&dep_regex).map_err(|_|FileCheckError::InvalidRegex)?));
                let expected_file_count = dep_filenames.len()*dep_multiplier;

                let use_sum = use_sum.unwrap_or(false);
                match use_sum {
                    false => {
                        let caps = re.captures(&matched_files[0]).ok_or(FileCheckError::ExpectedRegexMatch)?;
                        let cap_string = caps.get(1).ok_or(FileCheckError::RegexCaptureNotFound)?.as_str();
                        let from_name_int:usize = cap_string.parse().map_err(|_|FileCheckError::IntParseError)?;
                        expected_file_count *from_name_int
                    }
                    true => {
                        let mut sum = 0;
                        for file in &matched_files {
                            let cap = re.captures(file).ok_or(FileCheckError::ExpectedRegexMatch)?;
                            let cap_string = cap.get(1).ok_or(FileCheckError::RegexCaptureNotFound)?.as_str();
                            let to_add:usize = cap_string.parse().map_err(|_|FileCheckError::IntParseError)?;
                            sum = sum + to_add;
                        }
                        sum
                    }
                }


            }
            ListFile => 1,
            _=> 1
        };
        Ok(count)
    }
}
