mod ssh;
use ssh::session::{Session, Request, RequestType};
mod local;
use local::session::Session as LocalSession;
use tokio::fs;
use serde_json::{Value};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = get_matches();

    if let Some(matches) = matches.subcommand_matches("process") {
        let hosts: Value = serde_json::from_str(fs::read_to_string(matches.value_of("hosts").unwrap()).await?.as_str())?;
        let tasks: Value = serde_json::from_str(fs::read_to_string(matches.value_of("tasks").unwrap()).await?.as_str())?;

        let task_tags: Vec<&str> = tasks["tags"].as_array().unwrap().iter().map(|entry| entry.as_str().unwrap()).collect();

        for host in hosts["hosts"].as_array().unwrap() {
            let host_tags: Vec<&str> = host["tags"].as_array().unwrap().iter().map(|entry| entry.as_str().unwrap()).collect();

            match iters_have_common_entries(&task_tags, &host_tags) {
                true => println!("processing {}", host["title"]),
                false => println!("skipping {}", host["title"])
            }
        }
    }

    Ok(())
}

fn iters_have_common_entries(iter1: &Vec<&str>, iter2: &Vec<&str>) -> bool {
    for entry1 in iter1 {
        for entry2 in iter2 {
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
