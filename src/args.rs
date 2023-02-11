use std::path::PathBuf;

#[derive(clap::Parser,Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub action:Action
}

#[derive(clap::Args,Debug)]
pub struct ClientArgs {
    pub last_pipeline:String,
    pub runno_list:Vec<String>,
    #[clap(long)]
    pub base_runno:Option<String>,
    #[clap(short, long)]
    pub big_disk:Option<Vec<String>>,
    #[clap(short, long)]
    pub pipe_configs:Option<PathBuf>,
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