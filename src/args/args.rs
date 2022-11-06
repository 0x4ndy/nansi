use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub nansi_file: String,
}

impl Args {
    pub fn new() -> Result<Args, Box<dyn Error>> {
        Ok(Args::parse())
    }
}
