use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use regex::Regex;
use crate::stage::{Stage};
use serde::{Serialize,Deserialize};
use crate::args::ClientArgs;
use crate::host::Host;
use crate::request::{Request, Response};
use crate::status::{Status, StatusType};

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct PipeStatusConfig {
    pub label:String,
    pub preferred_computer:Option<String>,
    pub substitutions:SubstitutionTable,
    pub stages:Vec<Stage>,
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
        let s = utils::read_to_string(&file,"toml");
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
            stages: vec![
                Stage{
                    label: "make_header".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"inputs/.*nhdr"),
                    directory_pattern: "${BIGGUS_DISKUS}/co_reg_${PARAM0}-inputs".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "ants_registration".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/.*[Aa]ffine.(mat|txt)"),
                    directory_pattern: "${BIGGUS_DISKUS}/co_reg_${PARAM0}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "apply_transform".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"work/Reg_.*nhdr"),
                    directory_pattern: "${BIGGUS_DISKUS}/co_reg_${PARAM0}-work".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
            ],
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
            stages: vec![
                Stage{
                    label: "co_reg".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/co_reg.*headfile"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "make_4d".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/nii4D_[^_]+nii$"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_source".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"work/.*src(.gz)?$"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-work".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_fib".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/.*fib(.gz)?$"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_export".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/.*nii.gz"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: Some(vec!["qa" ,"iso", "fa", "ad", "rd"].iter().map(|thing| thing.to_string()).collect()),
                    file_counter: None
                },
            ],
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
            stages: vec![
                Stage{
                    label: "co_reg".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/co_reg.*headfile"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "make_4d".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/nii4D_[^_]+nii$"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "make_4d_nlsam".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: String::from(r"results/nii4D_[^_]+?NLSAM.nii$"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_source".to_string(),
                    preferred_computer: Some(String::from("delos")),
                    completion_file_pattern: String::from(r"work/.*src(.gz)?$"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-work".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_fib".to_string(),
                    preferred_computer: Some(String::from("delos")),
                    completion_file_pattern: String::from(r"results/.*fib(.gz)?$"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: None,
                    file_counter: None
                },
                Stage{
                    label: "dsi_studio_export".to_string(),
                    preferred_computer: Some(String::from("delos")),
                    completion_file_pattern: String::from("results/.*nii.gz"),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    required_file_keywords: Some(vec!["qa" ,"iso", "fa", "ad", "rd"].iter().map(|thing| thing.to_string()).collect()),
                    file_counter: None
                },
            ],
        };

        let s = toml::to_string_pretty(&co_reg).expect("cannot serialize pipe config!");
        utils::write_to_file(&dir.join("co_reg"),"toml",&s);

        let s = toml::to_string_pretty(&diffusion_calc).expect("cannot serialize pipe config!");
        utils::write_to_file(&dir.join("diffusion_calc"),"toml",&s);

        let s = toml::to_string_pretty(&diffusion_calc_nlsam).expect("cannot serialize pipe config!");
        utils::write_to_file(&dir.join("diffusion_calc_nlsam"),"toml",&s);

        println!("pipeline config templates generated in {:?}",dir);
    }

    pub fn required_servers(&self,pipe_name:&str) -> HashSet<String> {
        let pipe = self.get_pipe(pipe_name).expect("pipe not found!");
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

    pub fn pipe_status(&self,pipe_name:&str,args:&ClientArgs,ssh_connections:&mut HashMap<String,Host>, this_host:&String, big_disks:&Option<HashMap<String, String>>) -> (Vec<Status>,f32) {
        println!("running stage checks for {} ...",pipe_name);

        let pipe = self.get_pipe(pipe_name).expect("invalid pipeline name!");



        let mut pref_computer = pipe.preferred_computer.clone().unwrap_or(this_host.clone());

        let mut stage_statuses:Vec<Status> = vec![];

        for stage in &pipe.stages {

            //println!("building request for {} ...",stage.label);
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

            let host = ssh_connections.get_mut(&pref_computer).expect("host not found! what happened??");
            request.big_disk = match &big_disks {
                Some(disks) => {
                    match disks.get(&pref_computer) {
                        Some(disk) => Some(disk.to_owned()),
                        None => None
                    }
                }
                None => None
            };
            println!("sending request to {}",pref_computer);
            let stat = match host.submit_request(&request) {
                Response::Success(status) => status,
                Response::Error(_) => Status{
                    label: stage.label.clone(),
                    progress: StatusType::Invalid,
                    children: vec![]
                }
            };

            //println!("status received from {}",pref_computer);

            match &stat.progress {
                StatusType::NotStarted => { // if a pipe isn't started we have to consider it being a pipe

                    match self.get_pipe(&stage.label) {
                        Some(pipe) => {
                            let (children,progress) = self.pipe_status(&pipe.label,args,ssh_connections,this_host,big_disks);
                            let mut s = if progress == 0.0 {
                                Status{
                                    label: stage.label.clone(),
                                    progress: StatusType::NotStarted,
                                    children: vec![]
                                }
                            }else {
                                Status{
                                    label: stage.label.clone(),
                                    progress: StatusType::InProgress(progress),
                                    children: vec![]
                                }
                            };
                            s.children = children;
                            stage_statuses.push(s);
                        }
                        None => stage_statuses.push(stat)
                    }
                }
                _=> stage_statuses.push(stat)
            }
        }

        // integrate progress
        let mut total_progress:f32 = 0.0;
        stage_statuses.iter().for_each(|stat|{
            match &stat.progress {
                StatusType::Complete => total_progress = total_progress + 1.0,
                StatusType::InProgress(progress) => total_progress = total_progress + progress,
                _=> {}
            }
        });

        let frac_progress = total_progress /stage_statuses.len() as f32;

        (stage_statuses,frac_progress)
    }

}
