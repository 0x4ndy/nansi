use std::collections::HashMap;
use std::env;
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

/// Describes the structure and content of `NansiFile` file
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NansiFile {
    /// List of `exec` items
    pub exec_list: Vec<ExecItem>,

    /// Path to the `json` file based on which this struct was parsed
    #[serde(default = "default_as_empty_string")]
    pub file_path: String,
}

#[allow(dead_code)]
#[derive(PartialEq)]
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
            "{}\n{:?}",
            "The following aliases are duplicated which may cause issues with conditional execution:",
            duplicates
        )
        .to_string();

        print_warning(&msg);
    }

    let mut succ_label_list: Vec<&str> = Vec::new();

    for (idx, exec_item) in nansi_file.exec_list.iter().enumerate() {
        if !exec_meets_prerequisites(&exec_item, &succ_label_list) {
            let exec_status = ExecStatus::SKIP;
            if exec_item.print_status {
                print_status(&exec_item, idx + 1, exec_status);
            }

            let item_str = get_item_str(exec_item, idx);

            print_nominal(format!("Prerequisites for item {} are not met.", item_str).as_str());
            continue;
        }

        let (exec_status, output) = run_exec(&exec_item)?;

        if exec_status == ExecStatus::OK {
            if !exec_item.label.is_empty() && !succ_label_list.contains(&exec_item.label.as_str()) {
                succ_label_list.push(exec_item.label.as_str());
            }
        }

        if exec_item.print_status {
            print_status(&exec_item, idx + 1, exec_status);
        }

        if exec_item.print_output {
            print_nominal(&output);
        }
    }

    Ok(())
}

pub fn compile_arg(arg: &String) -> String {
    let mut compiled_arg = String::from(arg);

    let mut record = false;
    let mut tag = String::from("");
    let mut tags: Vec<String> = Vec::new();

    for (i, c) in arg.chars().enumerate() {
        match c {
            '{' => {
                if (i == 0)
                    || (i > 0
                        && arg.chars().nth(i - 1).unwrap() != '\\'
                        && arg.chars().nth(i - 1).unwrap() != '$')
                {
                    if record {
                        panic!("Incorrect number of {{");
                    } else {
                        record = true;
                    }
                }
            }
            '}' => {
                if (i == 0) || (i > 0 && arg.chars().nth(i - 1).unwrap() != '\\') {
                    if record {
                        record = false;
                        tags.push(tag.clone());
                        tag.clear();
                    }
                }
            }
            _ => {
                if record {
                    tag.push(c);
                }
            }
        }
    }

    for t in tags {
        let tag = format!("{{{t}}}");
        compiled_arg = compiled_arg.replace(tag.as_str(), env::var(t.as_str()).unwrap().as_str());
    }

    compiled_arg
}

fn run_exec(exec_item: &ExecItem) -> Result<(ExecStatus, String), Box<dyn Error>> {
    let mut exec_status = ExecStatus::ERR;
    let output: String;

    let mut args: Vec<String> = Vec::new();
    for arg in &exec_item.args {
        args.push(compile_arg(arg));
    }

    match Command::new(&exec_item.exec).args(&args).output() {
        Ok(result) => {
            if result.status.success() {
                exec_status = ExecStatus::OK;
            }

            output = if result.status.success() {
                String::from_utf8(result.stdout)?
            } else {
                String::from_utf8(result.stderr)?
            };
        }
        Err(e) => {
            exec_status = ExecStatus::ERR;
            output = e.to_string();
        }
    };

    Ok((exec_status, output))
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

    let mut keys: Vec<&str> = exec_map.keys().cloned().collect();
    keys.sort_by(|a, b| a.cmp(&b));

    keys
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
        ExecStatus::OK => String::from("OK").green().to_string(),
        ExecStatus::ERR => String::from("FAIL".red().to_string()),
        ExecStatus::WARN => String::from("WARN".yellow().to_string()),
        ExecStatus::SKIP => String::from("SKIP".dark_yellow().to_string()),
    };

    let item_str = get_item_str(exec_item, idx);

    println!(
        "[{}] {} {} {}",
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

#[test]
fn compile_arg_envvar_test() {
    let arg = String::from("cat Cargo.toml | grep \"version = \\\"${TEST}\\\"\"");

    env::set_var("TEST", "XYZ");

    let compiled_arg = compile_arg(&arg);
    assert_eq!(
        compiled_arg.as_str(),
        "cat Cargo.toml | grep \"version = \\\"${TEST}\\\"\""
    );
}

#[test]
fn compile_arg_var_test() {
    let arg = String::from("cat Cargo.toml | grep \"version = \\\"{TEST}\\\"\"");

    env::set_var("TEST", "XYZ");

    let compiled_arg = compile_arg(&arg);
    assert_eq!(
        compiled_arg.as_str(),
        "cat Cargo.toml | grep \"version = \\\"XYZ\\\"\""
    );
}
