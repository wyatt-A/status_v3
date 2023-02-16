use std::collections::HashMap;
use std::path::PathBuf;

#[derive(clap::Parser,Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub action:Action
}

#[derive(clap::Args,Debug)]
pub struct ClientArgs {
    pub base_runno:String,
    pub last_pipeline:String,
    #[clap(long)]
    pub runno_list:Vec<String>,
    #[clap(short, long)]
    pub big_disk:Option<Vec<String>>,
    #[clap(short, long)]
    pub pipe_configs:Option<PathBuf>,
    // #[clap(short, long)]
    // pub base_expand:Option<usize>,
}

#[derive(clap::Subcommand,Debug)]
pub enum Action {
    GenTemplates(GenTemplateArgs),
    Check(ClientArgs)
}

#[derive(clap::Args,Debug)]
pub struct GenTemplateArgs {
    #[clap(short, long)]
    pub directory:Option<PathBuf>
}

pub enum ArgParseError {
    BigDisk
}

impl ClientArgs {
    pub fn parse_big_disk(&self) -> Result<Option<HashMap<String, String>>,ArgParseError>{
        let big_disks = match &self.big_disk {
            Some(args) => {
                let mut big_disks = HashMap::<String,String>::new();
                for arg in args {
                    let split:Vec<&str> = arg.split(":").collect();
                    if split.len()  != 2 {
                        Err(ArgParseError::BigDisk)?
                    }
                    big_disks.insert(split[0].to_string(),split[1].to_string());
                }
                Some(big_disks)
            }
            None => None
        };
        Ok(big_disks)
    }
}