use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::{fs, io};

use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecItem {
    #[serde(default = "default_as_empty_string")]
    pub label: String,

    pub exec: String,

    #[serde(default = "default_as_empty_vec_string")]
    pub args: Vec<String>,

    #[serde(default = "default_as_true")]
    pub print_status: bool,

    #[serde(default = "default_as_false")]
    pub print_output: bool,

    #[serde(default = "default_as_empty_vec_string")]
    pub prerequisites: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NansiFile {
    pub exec_list: Vec<ExecItem>,

    #[serde(default = "default_as_empty_string")]
    pub file_path: String,
}

#[allow(dead_code)]
enum ExecStatus {
    OK,
    ERR,
    WARN,
    SKIP,
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

        let mut file: NansiFile = match serde_json::from_str(file_str.as_str()) {
            Ok(v) => v,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{}: {}", file_path, e.to_string()),
                ));
            }
        };

        file.file_path = String::from(file_path);

        Ok(file)
    }
}

pub fn execute(nansi_file: &NansiFile) -> Result<(), Box<dyn Error>> {
    print_nominal(
        format!("Using NansiFile: {}", nansi_file.file_path)
            .to_string()
            .as_str(),
    );

    let duplicates = get_label_duplicates(&nansi_file.exec_list);

    if duplicates.len() > 0 {
        let msg = format!(
            "{}\n{}",
            "The following aliases are duplicated which may cause issues with conditional execution:",
            duplicates.join("\n")
        )
        .to_string();

        print_warning(&msg);
    }

    let mut succ_label_list: Vec<&str> = Vec::new();

    for (idx, exec_item) in nansi_file.exec_list.iter().enumerate() {
        let mut exec_status = ExecStatus::ERR;

        if !exec_meets_prerequisites(&exec_item, &succ_label_list) {
            exec_status = ExecStatus::SKIP;
            if exec_item.print_status {
                print_status(&exec_item, idx + 1, exec_status);
            }

            let item_str = get_item_str(exec_item, idx);
            
            print_nominal(format!("Prerequisites for item {} are not met.", item_str).as_str());
            continue;
        }

        match Command::new(&exec_item.exec).args(&exec_item.args).output() {
            Ok(result) => {
                if result.status.success() {
                    exec_status = ExecStatus::OK;
                    if !exec_item.label.is_empty()
                        && !succ_label_list.contains(&exec_item.label.as_str())
                    {
                        succ_label_list.push(exec_item.label.as_str());
                    }
                }

                if exec_item.print_status {
                    print_status(&exec_item, idx + 1, exec_status);
                }

                if exec_item.print_output {
                    let output = if result.status.success() {
                        String::from_utf8(result.stdout)?
                    } else {
                        String::from_utf8(result.stderr)?
                    };
                    print_nominal(&output);
                }
            }
            Err(e) => {
                if exec_item.print_status {
                    print_status(exec_item, idx + 1, ExecStatus::ERR);
                }
                if exec_item.print_output {
                    print_nominal(e.to_string().as_str());
                }
            }
        };
    }

    Ok(())
}

fn get_label_duplicates(exec_list: &Vec<ExecItem>) -> Vec<&str> {
    let mut exec_map: HashMap<&str, u16> = HashMap::new();
    for exec in exec_list {
        if !exec.label.is_empty() {
            match exec_map.get(&exec.label.as_str()) {
                Some(count) => {
                    exec_map.insert(exec.label.as_str(), count + 1);
                }
                None => {
                    exec_map.insert(exec.label.as_str(), 1);
                }
            }
        }
    }
    exec_map.retain(|_, v| *v > 1);

    exec_map.keys().cloned().collect()
}

fn exec_meets_prerequisites(exec_item: &ExecItem, succ_label_list: &Vec<&str>) -> bool {
    for prereq in &exec_item.prerequisites {
        if !succ_label_list.contains(&prereq.as_str()) {
            return false;
        }
    }

    true
}

fn get_item_str(exec_item: &ExecItem, idx: usize) -> String {
    let item_str = if exec_item.label.is_empty() {
        String::from(format!("[{}]", idx.to_string()))
    } else {
        String::from(format!("[{}][{}]", idx.to_string(), &exec_item.label))
    };

    item_str
}

fn print_status(exec_item: &ExecItem, idx: usize, exec_status: ExecStatus) {
    let status = match exec_status {
        ExecStatus::OK => String::from("[OK]").green().to_string(),
        ExecStatus::ERR => String::from("[FAIL]".red().to_string()),
        ExecStatus::WARN => String::from("[WARN]".yellow().to_string()),
        ExecStatus::SKIP => String::from("[SKIP]".dark_yellow().to_string()),
    };

    let item_str = get_item_str(exec_item, idx);

    println!(
        "{} {} {} {}",
        status,
        item_str,
        exec_item.exec,
        exec_item.args.join(" ")
    );
}

#[allow(dead_code)]
fn print_nominal(msg: &str) {
    println!("{}", msg);
}

#[allow(dead_code)]
fn print_ok(msg: &str) {
    println!("{} {}", "[OK]", msg);
}

#[allow(dead_code)]
fn print_warning(msg: &str) {
    println!("{} {}", "[WARN]".yellow().to_string(), msg);
}

#[allow(dead_code)]
fn print_error(msg: &str) {
    println!("{} {}", "[ERR]".red().to_string(), msg);
}

fn default_as_false() -> bool {
    false
}

fn default_as_true() -> bool {
    true
}

fn default_as_empty_vec_string() -> Vec<String> {
    vec![]
}

fn default_as_empty_string() -> String {
    String::from("")
}
