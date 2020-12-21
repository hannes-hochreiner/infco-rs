mod ssh;
use ssh::ssh_service;
mod local;
use local::local_service;
use tokio::fs;
use serde_json::{Value};
mod service;
use service::Service;
mod error;
use error::InfcoError;
use log::{debug, error, info};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let matches = get_matches();

    if let Some(matches) = matches.subcommand_matches("process") {
        let hosts: Value = serde_json::from_str(fs::read_to_string(matches.value_of("hosts").unwrap()).await?.as_str())?;
        let tasks: Value = serde_json::from_str(fs::read_to_string(matches.value_of("tasks").unwrap()).await?.as_str())?;

        let task_tags: Vec<&str> = tasks["tags"].as_array().unwrap().iter().map(|entry| entry.as_str().unwrap()).collect();

        for host in hosts["hosts"].as_array().unwrap() {
            let host_tags: Vec<&str> = host["tags"].as_array().unwrap().iter().map(|entry| entry.as_str().unwrap()).collect();

            match vecs_have_common_entries(&task_tags, &host_tags) {
                true => {
                    info!("processing host {}", host["title"]);
                    process_tasks_for_host(&tasks["tasks"].as_array().unwrap(), host).await?;
                },
                false => info!("skipping host {}", host["title"])
            }
        }
    }

    Ok(())
}

async fn process_tasks_for_host(tasks: &Vec<Value>, host: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut context: Box<dyn Service> = match host["context"]["type"].as_str() {
        Some("ssh") => {
            let host_name = match host["context"]["config"]["host"].as_str() {
                Some(s) => s,
                None => return Err(Box::new(InfcoError::new(String::from("no host specified"))))
            };
            let user_name = match host["context"]["config"]["username"].as_str() {
                Some(s) => s,
                None => return Err(Box::new(InfcoError::new(String::from("no user specified"))))
            };
            Box::new(ssh_service::SshService::new(host_name.into(), user_name.into())?)
        },
        Some("local") => Box::new(local_service::LocalService::new()?),
        Some(name) => return Err(Box::new(InfcoError::new(format!("unknown context type \"{}\"", name)))),
        None => return Err(Box::new(InfcoError::new(String::from("no context type found"))))
    };

    for task in tasks {
        info!("task {} ({})", task["title"], task["type"]);
        match task["type"].as_str() {
            Some("exec") => {
                context.run(task["config"]["command"].as_str().unwrap().to_string()).await?;
            },
            Some(name) => return Err(Box::new(InfcoError::new(format!("unknown task type \"{}\"", name)))),
            None => return Err(Box::new(InfcoError::new(String::from("no task type found"))))
        }
    }

    Ok(())
}

#[test]
fn function_vecs_have_common_entries() {
    assert_eq!(vecs_have_common_entries(&vec!["test1"], &vec!["test2"]), false);
    assert_eq!(vecs_have_common_entries(&vec!["test1", "test2", "test3"], &vec!["test4", "test5", "test6"]), false);
    assert_eq!(vecs_have_common_entries(&vec!["test1"], &vec!["test1"]), true);
    assert_eq!(vecs_have_common_entries(&vec!["test1", "test2", "test3"], &vec!["test4", "test2", "test6"]), true);
}

fn vecs_have_common_entries(vec1: &Vec<&str>, vec2: &Vec<&str>) -> bool {
    for entry1 in vec1 {
        for entry2 in vec2 {
            if entry1 == entry2 {
                return true;
            }
        }
    }

    false
}

fn get_matches() -> clap::ArgMatches<'static> {
    use clap::{Arg, App, SubCommand};

    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(SubCommand::with_name("process")
            .about("process a combination of task and host files")
            .arg(Arg::with_name("hosts")
                .short("h")
                .takes_value(true)
                .required(true)
                .help("host file"))
            .arg(Arg::with_name("tasks")
                .short("t")
                .takes_value(true)
                .required(true)
                .help("task file")))
        .get_matches()
}
