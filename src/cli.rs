use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]

pub struct App {
    #[arg(short, long)]
    pub filetypes: Option<String>,

    #[arg(short, long)]
    pub dry: bool,

    #[arg(short, long)]
    pub dir: Option<PathBuf>,

}