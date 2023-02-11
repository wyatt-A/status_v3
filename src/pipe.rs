use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use regex::Regex;
use crate::stage::{SignatureType, Stage};
use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct PipeStatusConfig {
    pub label:String,
    pub preferred_computer:Option<String>,
    pub stages:Vec<Stage>,
}

impl PipeStatusConfig {
    pub fn get_stages(&self,conf_collection:&ConfigCollection) -> Vec<Stage> {
        let mut stages_flat = vec![];
        for stage in &self.stages {
            match conf_collection.get_pipe(&stage.label) {
                Some(pipe) =>{
                    let mut stages = pipe.get_stages(&conf_collection);
                    stages_flat.append(&mut stages);
                },
                None =>{
                    stages_flat.push(stage.clone());
                }
            }
        }
        stages_flat
    }

    pub fn from_file(file:&Path) -> Result<Self,ConfigCollectionError> {
        let s = utils::read_to_string(&file,"toml");
        Ok(toml::from_str(&s).map_err(|e|ConfigCollectionError::ConfigParse(file.clone().to_owned(),e))?)
    }
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ConfigCollection {
    configs:HashMap<String,PipeStatusConfig>
}

pub enum ConfigCollectionError {
    NoConfigsFound,
    ConfigParse(PathBuf,toml::de::Error)
}



impl ConfigCollection {

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
            stages: vec![
                Stage{
                    label: "make_header".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"inputs/.*nhdr").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/co_reg_${PARAM0}-inputs".to_string(),
                    signature_type: SignatureType::ManyToMany,
                    required_file_keywords: None,
                },
                Stage{
                    label: "ants_registration".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"results/.*[Aa]ffine.(mat|txt)").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/co_reg_${PARAM0}-results".to_string(),
                    signature_type: SignatureType::ManyToMany,
                    required_file_keywords: None,
                },
                Stage{
                    label: "apply_transform".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"work/Reg_.*nhdr").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/co_reg_${PARAM0}-work".to_string(),
                    signature_type: SignatureType::ManyToMany,
                    required_file_keywords: None,
                },
            ],
        };
        let diffusion_calc = PipeStatusConfig{
            label: "diffusion_calc".to_string(),
            preferred_computer: Some(String::from("delos")),
            stages: vec![
                Stage{
                    label: "co_reg".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"results/co_reg.*headfile").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    signature_type: SignatureType::ManyToOne,
                    required_file_keywords: None,
                },
                Stage{
                    label: "make_4d".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"results/nii4D_[^_]+nii$").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    signature_type: SignatureType::ManyToOne,
                    required_file_keywords: None,
                },
                Stage{
                    label: "dsi_studio_source".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"work/.*src(.gz)?$").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-work".to_string(),
                    signature_type: SignatureType::OneToOne,
                    required_file_keywords: None,
                },
                Stage{
                    label: "dsi_studio_fib".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"results/.*fib(.gz)?$").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    signature_type: SignatureType::OneToOne,
                    required_file_keywords: None,
                },
                Stage{
                    label: "dsi_studio_export".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"results/.*nii.gz").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    signature_type: SignatureType::OneToMany,
                    required_file_keywords: Some(vec!["qa" ,"iso", "fa", "ad", "rd"].iter().map(|thing| thing.to_string()).collect()),
                },
            ],
        };
        let diffusion_calc_nlsam = PipeStatusConfig{
            label: "diffusion_calc_nlsam".to_string(),
            preferred_computer: Some(String::from("civmcluster1")),
            stages: vec![
                Stage{
                    label: "co_reg".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"results/co_reg.*headfile").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    signature_type: SignatureType::ManyToOne,
                    required_file_keywords: None,
                },
                Stage{
                    label: "make_4d".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"results/nii4D_[^_]+nii$").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    signature_type: SignatureType::ManyToOne,
                    required_file_keywords: None,
                },
                Stage{
                    label: "make_4d_nlsam".to_string(),
                    preferred_computer: None,
                    completion_file_pattern: Regex::new(r"results/nii4D_[^_]+?NLSAM.nii$").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    signature_type: SignatureType::ManyToOne,
                    required_file_keywords: None,
                },
                Stage{
                    label: "dsi_studio_source".to_string(),
                    preferred_computer: Some(String::from("delos")),
                    completion_file_pattern: Regex::new(r"work/.*src(.gz)?$").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-work".to_string(),
                    signature_type: SignatureType::OneToOne,
                    required_file_keywords: None,
                },
                Stage{
                    label: "dsi_studio_fib".to_string(),
                    preferred_computer: Some(String::from("delos")),
                    completion_file_pattern: Regex::new(r"results/.*fib(.gz)?$").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    signature_type: SignatureType::OneToOne,
                    required_file_keywords: None,
                },
                Stage{
                    label: "dsi_studio_export".to_string(),
                    preferred_computer: Some(String::from("delos")),
                    completion_file_pattern: Regex::new(r"results/.*nii.gz").unwrap(),
                    directory_pattern: "${BIGGUS_DISKUS}/${PREFIX}${SEP}${BASE}NLSAM${SEP}${PROGRAM}${SEP}${SUFFIX}-results".to_string(),
                    signature_type: SignatureType::OneToMany,
                    required_file_keywords: Some(vec!["qa" ,"iso", "fa", "ad", "rd"].iter().map(|thing| thing.to_string()).collect()),
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

    pub fn from_dir(dir:&Path) -> Result<Self,ConfigCollectionError> {
        let mut configs = HashMap::<String,PipeStatusConfig>::new();
        match utils::find_files(dir,"toml",true) {
            Some(files) => {
                for file in files {
                    let cfg = PipeStatusConfig::from_file(&file)?;
                    //let cfg:PipeStatusConfig = toml::from_str(&toml_str).expect("unable to load config!");
                    configs.insert(cfg.label.clone(),cfg);
                }
                Ok(ConfigCollection{configs})
            },
            None => Err(ConfigCollectionError::NoConfigsFound)
        }
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

}
