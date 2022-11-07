use std::{fs, io};
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;

use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecItem {
    #[serde(default = "default_label")]
    pub label: String,

    pub exec: String,

    #[serde(default = "default_as_no_args")]
    pub args: Vec<String>,
    
    #[serde(default = "default_as_true")]
    pub print_status: bool,

    #[serde(default = "default_as_false")]
    pub print_output: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NansiFile {
    pub exec_list: Vec<ExecItem>,
}

enum ExecStatus {
    OK,
    ERR
}

impl NansiFile {
    pub fn from(file_path: &str) -> Result<NansiFile, io::Error> {
        let file_str = match fs::read_to_string(file_path) {
            Ok(v) => v,
            Err(e) => {
                return Err(io::Error::new(
                    e.kind(),
                    format!("{}: {}", file_path, e.to_string()),
                ));
            }
        };

        let file: NansiFile = match serde_json::from_str(file_str.as_str()) {
            Ok(v) => v,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{}: {}", file_path, e.to_string()),
                ));
            }
        };

        Ok(file)
    }
}

pub fn execute(nansi_file: &NansiFile) -> Result<(), Box<dyn Error>>{
    let duplicates = get_label_duplicates(&nansi_file.exec_list);
    
    if duplicates.len() > 0 {
        print_duplicates_warn(&duplicates);
    }


    for exec_item in &nansi_file.exec_list {
        match Command::new(&exec_item.exec).args(&exec_item.args).output() {
            Ok(result) => {
                if exec_item.print_status {
                    let exec_status = if result.status.success() {
                        ExecStatus::OK
                    } else {
                        ExecStatus::ERR
                    };
                    print_status(&exec_item, exec_status);
                }

                if exec_item.print_output {
                    let output = if result.status.success() {
                        String::from_utf8(result.stdout)?
                    } else {
                        String::from_utf8(result.stderr)?
                    };
                    print_output(&output);
                }
            },
            Err(e) => {
                if exec_item.print_status {
                    print_status(exec_item, ExecStatus::ERR);
                }
                if exec_item.print_output {
                    print_output(e.to_string().as_str());
                }
            }
        };
    }

    Ok(())
}

fn print_output(output: &str) {
    println!("{}", output);   
}

fn print_status(exec_item: &ExecItem, exec_status: ExecStatus) {
    let status = match exec_status {
        ExecStatus::OK => String::from("[OK]").green().to_string(),
        ExecStatus::ERR => String::from("[FAIL]".red().to_string())
    };

    println!("{} {} {}", status, exec_item.exec, exec_item.args.join(" "));
}

fn print_duplicates_warn(duplicates: &Vec<&str>) {
    println!("{} {}\n{}", 
             "[WARN]".yellow(), 
             "The following aliases are duplicated which may cause issues with conditional execution:", 
             duplicates.join("\n")
            );
}

fn default_as_false() -> bool {
    false
}

fn default_as_true() -> bool {
   true 
}

fn default_as_no_args() -> Vec<String> {
    vec![]
}

fn default_label() -> String {
    String::from("")
}

fn get_label_duplicates(exec_list: &Vec<ExecItem>) -> Vec<&str> {
    let mut exec_map: HashMap<&str, u16> = HashMap::new();
    for exec in exec_list {
        if !exec.label.is_empty() {
            match exec_map.get(&exec.label.as_str()) {
                Some(count) => { exec_map.insert(exec.label.as_str(), count + 1); }
                None => { exec_map.insert(exec.label.as_str(), 1); }
            }
        }
    }
    exec_map.retain(|_, v| *v > 1);

    exec_map.keys().cloned().collect()
}

