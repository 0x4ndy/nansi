mod args;
mod exec;

use std::error::Error;

use args::Args;

pub fn run() -> Result<(), Box<dyn Error>> {
    
    let args = match Args::new() {
        Ok(args) => args,
        Err(e) => {
            return Err(e);
        }
    };

    let nansi_file = exec::NansiFile::from(args.nansi_file.as_str())?;
    exec::execute(&nansi_file)?;

    Ok(())

}
