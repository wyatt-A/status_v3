use std::collections::{HashMap, HashSet};
use std::path::Path;
use crate::stage::Stage;
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
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct ConfigCollection {
    configs:HashMap<String,PipeStatusConfig>
}

impl ConfigCollection {

    pub fn from_dir(dir:&Path) -> Self {
        let mut configs = HashMap::<String,PipeStatusConfig>::new();
        match utils::find_files(dir,"toml",true) {
            Some(files) => {
                for file in files {
                    let toml_str = utils::read_to_string(&file,"toml");
                    let cfg:PipeStatusConfig = toml::from_str(&toml_str).expect("unable to load config!");
                    configs.insert(cfg.label.clone(),cfg);
                }
                ConfigCollection{configs}
            },
            None => panic!("no config files found!")
        }
    }

    // pub fn _servers(&self) -> HashSet<String> {
    //
    //     let mut servers = HashSet::<String>::new();
    //
    //     for (_,cfg) in &self.configs {
    //         match &cfg.preferred_computer {
    //             Some(computer) => {servers.insert(computer.clone());}
    //             None => {}
    //         }
    //         for stage in &cfg.stages {
    //             match &stage.preferred_computer {
    //                 Some(computers) => {
    //                     for computer in computers {
    //                         servers.insert(computer.clone());
    //                     }
    //                 }
    //                 None => {}
    //             }
    //         }
    //     }
    //     servers
    // }

    // pub fn servers(&self,pipe_name:&str) -> HashSet<String> {
    //     let mut servers = HashSet::<String>::new();
    //
    //     let pipe = self.get_pipe(pipe_name).unwrap();
    //
    //     match &pipe.preferred_computer {
    //         Some(computer) => {servers.insert(computer.clone());}
    //         None => {}
    //     }
    //
    //     let stages = pipe.get_stages(&self);
    //     for stage in &stages {
    //         match &stage.preferred_computer {
    //             Some(computers) => {
    //                 for computer in computers {
    //                     servers.insert(computer.clone());
    //                 }
    //             }
    //             None => {}
    //         }
    //     }
    //     servers
    // }

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

// // get all stages
// pub fn get_pipe_servers(pipe:&PipeStatusConfig,pipe_collection:&ConfigCollection,server_list:&mut HashSet<String>) {
//     match &pipe.preferred_computer {
//         Some(computer) => {server_list.insert(computer.clone());}
//         _=> {}
//     }
//     for stage in &pipe.stages {
//          match &stage.preferred_computer {
//              Some(computer) => {server_list.insert(computer.clone());}
//              _=> {}
//          }
//         match pipe_collection.get_pipe(&stage.label) {
//             Some(pipe) =>{
//                 let mut pipe = pipe.clone();
//                 pipe.preferred_computer = stage.preferred_computer.clone();
//                 get_pipe_servers(&pipe,pipe_collection,server_list)
//             },
//             _=> {}
//         }
//     }
// }
























