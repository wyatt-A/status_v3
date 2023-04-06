use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use regex::Regex;
use civm_rust_utils as utils;
use crate::stage::{FileCheckError, FileCounter, Stage};
use serde::{Serialize,Deserialize};
use crate::args::ClientArgs;
use crate::host::{Host, RemoteHost};
use crate::request::{Request, Response};
use crate::status::{Status, StatusType};

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct PipeStatusConfig {
    pub label:String,
    pub preferred_computer:Option<String>,
    pub substitutions:SubstitutionTable,
    pub pipeline_headfile:Option<PipelineHeadfile>,
    pub stages:Vec<Stage>,
    pub archive:Option<ArchiveCheckParams>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct PipelineHeadfile {
    completion_file_pattern:String,
    directory_pattern:String,
    file_counter:Option<FileCounter>,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ArchiveCheckParams {
    pub preferred_computer:Option<String>,
    pub matches_headfile:bool,
}


#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct SubstitutionTable {
    pub base:Option<String>,
    pub param0:Option<String>,
    pub prefix:Option<String>,
    pub sep:Option<String>,
    pub program:Option<String>,
    pub suffix:Option<String>,
}

impl SubstitutionTable {
    pub fn to_hash(&self) -> HashMap<String,String> {
        let mut h = HashMap::<String,String>::new();

        match &self.prefix {
            Some(prefix) => {
                h.insert(String::from("PREFIX"),prefix.clone());
            }
            None => {}
        }

        match &self.sep {
            Some(sep) => {
                h.insert(String::from("SEP"),sep.clone());
            }
            None => {}
        }

        match &self.program {
            Some(program) => {
                h.insert(String::from("PROGRAM"),program.clone());
            }
            None => {}
        }

        match &self.suffix {
            Some(suffix) => {
                h.insert(String::from("SUFFIX"),suffix.clone());
            }
            None => {}
        }

        match &self.base {
            Some(base) => {
                h.insert(String::from("BASE"),base.clone());
            }
            None => {}
        }

        match &self.param0 {
            Some(param0) => {
                h.insert(String::from("PARAM0"),param0.clone());
            }
            None => {}
        }

        h

    }
}


impl PipeStatusConfig {
    pub fn from_file(file:&Path) -> Result<Self,ConfigCollectionError> {
        let s = utils::read_to_string(&file,Some("toml"));
        Ok(toml::from_str(&s).map_err(|e|ConfigCollectionError::ConfigParse(file.clone().to_owned(),e))?)
    }
    pub fn sub_table(&self) -> SubstitutionTable {
        self.substitutions.clone()
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ConfigCollection {
    configs:HashMap<String,PipeStatusConfig>
}

#[derive(Debug,Clone)]
pub enum ConfigCollectionError {
    NoConfigsFound,
    ConfigParse(PathBuf,toml::de::Error)
}

impl ConfigCollection {

    pub fn from_dir(dir:&Path) -> Result<Self,ConfigCollectionError> {
        let mut configs = HashMap::<String,PipeStatusConfig>::new();
        match utils::find_files(dir,"toml",true) {
            Some(files) => {
                for file in files {
                    let cfg = PipeStatusConfig::from_file(&file)?;
                    configs.insert(cfg.label.clone(),cfg);
                }
                Ok(ConfigCollection{configs})
            },
            None => Err(ConfigCollectionError::NoConfigsFound)
        }
    }

    pub fn generate_templates(dir:&Path) {

        if dir.exists(){
            println!("directory already exists! Nothing will be generated.");
            return
        }

        match std::fs::create_dir_all(dir) {
            Err(e) => {
                println!("cannot make directory {:?}. {:?}",dir,e);
                return
            } ,
            Ok(_) => {}
        }

        let co_reg = PipeStatusConfig{
            label: String::from("co_reg"),
            preferred_computer: Some(String::from("civmcluster1")),
            substitutions: SubstitutionTable {
                base: None,
                param0: None,
                prefix: None,
                sep: None,
                program: None,
                suffix: None
            },
            pipeline_headfile: None,
            stages: vec![
                Stage{
                    label: "make_header".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"inputs/.*nhdr"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/co_reg_${PARAM0}-inputs".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "ants_registration".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/.*[Aa]ffine.(mat|txt)"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/co_reg_${PARAM0}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "apply_transform".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"work/Reg_.*nhdr"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/co_reg_${PARAM0}-work".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
            ],
            archive: None
        };
        let diffusion_calc = PipeStatusConfig{
            label: "diffusion_calc".to_string(),
            preferred_computer: Some(String::from("delos")),
            substitutions: SubstitutionTable {
                base: None,
                param0: None,
                prefix: None,
                sep: None,
                program: None,
                suffix: None
            },
            pipeline_headfile: None,
            stages: vec![
                Stage{
                    label: "co_reg".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/co_reg.*headfile"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "make_4d".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/nii4D_[^_]+nii$"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_source".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"work/.*src(.gz)?$"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-work".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_fib".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/.*fib(.gz)?$"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_export".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/.*nii.gz"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: Some(vec!["qa" ,"iso", "fa", "ad", "rd"].iter().map(|thing| thing.to_string()).collect()),
                    file_counter: None
                },
            ],
            archive: None
        };
        let diffusion_calc_nlsam = PipeStatusConfig{
            label: "diffusion_calc_nlsam".to_string(),
            preferred_computer: Some(String::from("civmcluster1")),
            substitutions: SubstitutionTable {
                base: None,
                param0: None,
                prefix: None,
                sep: None,
                program: None,
                suffix: None
            },
            pipeline_headfile: None,
            stages: vec![
                Stage{
                    label: "co_reg".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/co_reg.*headfile"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "make_4d".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/nii4D_[^_]+nii$"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "make_4d_nlsam".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/nii4D_[^_]+?NLSAM.nii$"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_source".to_string(),
                    preferred_computer: Some(String::from("delos")),
                    completion_file_pattern: String::from(r"work/.*src(.gz)?$"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-work".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_fib".to_string(),
                    preferred_computer: Some(String::from("delos")),
                    completion_file_pattern: String::from(r"results/.*fib(.gz)?$"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_export".to_string(),
                    preferred_computer: Some(String::from("delos")),
                    completion_file_pattern: String::from("results/.*nii.gz"),
                    weighting: None,
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: Some(vec!["qa" ,"iso", "fa", "ad", "rd"].iter().map(|thing| thing.to_string()).collect()),
                    file_counter: None
                },
            ],
            archive: None
        };

        let s = toml::to_string_pretty(&co_reg).expect("cannot serialize pipe config!");
        utils::write_to_file(&dir.join("co_reg"),Some("toml"),&s);

        let s = toml::to_string_pretty(&diffusion_calc).expect("cannot serialize pipe config!");
        utils::write_to_file(&dir.join("diffusion_calc"),Some("toml"),&s);

        let s = toml::to_string_pretty(&diffusion_calc_nlsam).expect("cannot serialize pipe config!");
        utils::write_to_file(&dir.join("diffusion_calc_nlsam"),Some("toml"),&s);

        println!("pipeline config templates generated in {:?}",dir);
    }

    pub fn required_servers(&self,pipe_name:&str) -> HashSet<String> {
        let pipe = self.get_pipe(pipe_name).expect(
            &format!("{} not found in pipe configurations!",&pipe_name) );
        let mut servers = HashSet::<String>::new();
        self.get_pipe_servers(&pipe,&mut servers);
        servers
    }

    fn get_pipe_servers(&self,pipe:&PipeStatusConfig,server_list:&mut HashSet<String>) {
        match &pipe.preferred_computer {
            Some(computer) => {server_list.insert(computer.clone());}
            _=> {}
        }
        for stage in &pipe.stages {
            match &stage.preferred_computer {
                Some(computer) => {server_list.insert(computer.clone());}
                _=> {}
            }
            match self.get_pipe(&stage.label) {
                Some(pipe) =>{
                    let mut pipe = pipe.clone();
                    pipe.preferred_computer = stage.preferred_computer.clone();
                    self.get_pipe_servers(&pipe,server_list)
                },
                _=> {}
            }
        }
    }

    pub fn get_pipe(&self,pipe_name:&str) -> Option<&PipeStatusConfig> {
        self.configs.get(pipe_name)
    }

    pub fn is_pipe_output_archived(&self,pipe_name:&str,args:&ClientArgs,ssh_connections:&mut HashMap<String, Host>,this_host:&String) -> bool {

        let pipe = self.get_pipe(pipe_name).expect("invalid pipeline name!");
        let archive_settings = pipe.archive.clone().unwrap();

        if !archive_settings.matches_headfile {
            panic!("get name of thing from headfile not yet implemented");
        }

        let preferred_computer = archive_settings.preferred_computer.unwrap_or(this_host.clone());
        let host = ssh_connections.get_mut(&preferred_computer).expect(&format!("host {} not found",preferred_computer));

        let file_pattern = pipe.pipeline_headfile.clone().expect("pipeline headfile must be defined").completion_file_pattern.clone();

        let mut subs = pipe.substitutions.to_hash();

        if !args.runno_list.is_empty() {
            subs.insert(String::from("PARAM0"),args.runno_list[0].to_string());
        }
        subs.insert(String::from("BASE"),args.base_runno.clone());
        let resolved_filename = substitute(&file_pattern,&subs).expect("substitions failed");

        let filename =Path::new(&resolved_filename).with_extension("");
        let filename = filename.file_name().unwrap();
        host.civm_in_db(&vec![filename.to_string_lossy().to_string()])
    }

    pub fn pipe_status(&self, pipe_name:&str, args:&ClientArgs, ssh_connections:&mut HashMap<String, Host>, this_host:&String, big_disks:&Option<HashMap<String, String>>) -> (Status) {
        println!("running stage checks for {} ...",pipe_name);

        let pipe = self.get_pipe(pipe_name).expect(&format!("invalid pipeline name! {} not found",&pipe_name) );

        let mut pref_computer = pipe.preferred_computer.clone().unwrap_or(this_host.clone());

        let mut pipe_status = Status{
            label: pipe.label.clone(),
            progress: StatusType::NotStarted,
            children: vec![]
        };

        /*
        // look for a stage labeled archive to check first
        let archive_stage = pipe.stages.iter().find_map(|stage|{
            match stage.label.as_str(){
                "archive" => Some(stage.clone()),
                 _=> None
            }
        });

        // check the archive stage if it exists. If it is complete, skip further checks and return
        let archive_status = match &archive_stage {
            Some(stage) => {
                let stat = stage_stat(&pref_computer,&pipe,&stage,args,ssh_connections,big_disks);
                match stat.progress {
                    StatusType::Complete => {
                        pipe_status.progress=StatusType::Complete;
                        pipe_status.children=vec![stat];
                        return pipe_status
                    }
                    _=> {}
                }
                Some(stat)
            },
            None => {None}
        };
        */
        // look for a stage labeled archive to check first
        let archive_status = pipe.stages.iter().find_map(|stage| {
            match stage.label.as_str() {
                "archive" => {
                    let stat = stage_stat(&pref_computer, &pipe, &stage, args, ssh_connections, big_disks);
                    match stat.progress {
                        StatusType::Complete => {
                            pipe_status.progress = StatusType::Complete;
                            pipe_status.children = vec![stat.clone()]
                        }
                        _ => {}
                    }
                    Some(stat)
                }
                _ => { None }
            }
        });

        match pipe_status.progress {
            StatusType::Complete => {
                return pipe_status;
            }
            _ => {}
        }

        let mut stage_statuses:Vec<Status> = vec![];
        // stage sum and divisor are used when integrating stage progress to account for different weighting.
        // the stage progress is multiplied by the weighting, the divisor is the sum of the weighting.
        // this allows progress to remain fractional 0 to 1
        let mut stage_sum:f32 = 0.0;
        let mut stage_divisor = 0.0;
        for stage in &pipe.stages {
            // set stage weight
            let mut stage_weight = match stage.weighting {
                Some(weight) => {
                    weight
                }
                _=> { 1.0 }
            };
            // collect status
            let mut stat = match stage.label.as_str() {
                "archive" => {
                    // this will exclude archive from the status, meaning we'll sho 100% complete prior to archiving.
                    stage_weight = 0.0;
                    // we already checked the archive stage first above, and it couldn't be done because it would have returned.
                    archive_status.clone().unwrap()
                }
                _=> {
                    stage_stat(&pref_computer, &pipe, stage, args, ssh_connections, big_disks)
                }
            };
            match &stat.progress {
                StatusType::NotStarted => { // if a stage isn't started we have to consider it being a pipe
                    match self.get_pipe(&stage.label) {
                        Some(pipe) => {
                            stat= self.pipe_status(&pipe.label,args,ssh_connections,this_host,big_disks);
                        }
                        None => {}
                    }
                }
                _=> { }
            }
            match &stat.progress {
                StatusType::Invalid(e) => {
                    // todo Should better handle invalids, for now forcing 0 progress...
                    stat.progress = StatusType::InProgress(0.0);
                }
                _ => {}
            }
            stage_sum = stage_sum + stat.progress.to_float() * stage_weight;
            stage_divisor = stage_divisor + stage_weight;
            stage_statuses.push(stat)
            //println!("building request for {} ...",stage.label);
            // let mut request = Request{
            //     sub_table: pipe.sub_table(),
            //     stage: stage.clone(),
            //     big_disk:None,
            //     run_number_list:args.runno_list.clone(),
            //     base_runno:args.base_runno.clone(),
            // };
            // // overwrite preferred computer if needed
            // match &stage.preferred_computer {
            //     Some(computer) => {pref_computer = computer.clone()}
            //     None => {}
            // }
            // request.big_disk = match &big_disks {
            //     Some(disks) => {
            //         match disks.get(&pref_computer) {
            //             Some(disk) => Some(disk.to_owned()),
            //             None => None
            //         }
            //     }
            //     None => None
            // };
            // let host = ssh_connections.get_mut(&pref_computer).expect("host not found! what happened??");
            // println!("sending request to {}",pref_computer);
            // let stat = match host.submit_request(&request) {
            //     Response::Success(status) => status,
            //     Response::Error(_) => Status{
            //         label: stage.label.clone(),
            //         progress: StatusType::Invalid,
            //         children: vec![]
            //     }
            // };
            //println!("status received from {}",pref_computer);
        }
        // pack progress and children into our pipe status
        pipe_status.progress=StatusType::InProgress( stage_sum / stage_divisor );
        pipe_status.children=stage_statuses;
        pipe_status
    }

}

fn stage_stat(pref_computer:&String, pipe:&PipeStatusConfig,stage:&Stage, args:&ClientArgs, ssh_connections:&mut HashMap<String, Host>, big_disks:&Option<HashMap<String, String>>) -> Status {
    let mut pref_computer = pref_computer.clone();
    let mut request = Request{
        sub_table: pipe.sub_table(),
        stage: stage.clone(),
        big_disk:None,
        run_number_list:args.runno_list.clone(),
        base_runno:args.base_runno.clone(),
    };
    // overwrite preferred computer if needed
    match &stage.preferred_computer {
        Some(computer) => {pref_computer = computer.clone()}
        None => {}
    }
    request.big_disk = match &big_disks {
        Some(disks) => {
            match disks.get(pref_computer.as_str()) {
                Some(disk) => Some(disk.to_owned()),
                None => None
            }
        }
        None => None
    };
    let host = ssh_connections.get_mut(&pref_computer).expect("host not found! what happened??");
    println!("\tto:{}",pref_computer);
    let stat = match host.submit_request(&request) {
        Response::Success(status) => status,
        Response::Error(e) => Status{
            label: stage.label.clone(),
            progress: StatusType::Invalid(e.clone()),
            children: vec![]
        }
    };
    stat
}

pub fn substitute(thing_to_resolve: &String, sub_table:&HashMap<String,String>) -> Result<String,FileCheckError> {
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
