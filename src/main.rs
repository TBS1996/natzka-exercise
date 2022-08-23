
use clap::Parser;
use serde_derive::Deserialize;
use std::{
    path::PathBuf,
    process::{
        Command,
        ExitStatus,
        Stdio,
        Child,
    },
    ffi::OsStr,
};


#[derive(Debug, Deserialize, Clone, PartialEq)]
enum RestartCondition{
    Never,
    OnError,
    Always,
}

#[derive(Deserialize, Clone)]
struct ProcessConfig{
    name: String,
    path: String,
    arguments: Vec<String>,
    non_error_codes: Vec<i8>,
    restart_condition: RestartCondition,
}

#[derive(Deserialize)]
struct Tasks{
    task: Vec<ProcessConfig>
}

struct ChildAndConfig{
    child: Child, 
    config: ProcessConfig,
    prev_exit_status: Option<ExitStatus>
}

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Debug)]
struct Cli {
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}


#[derive(PartialEq)]
enum MajorEvents{
    Started,
    ErrStopped,
    OkStopped,
    Restarted,
}



fn get_path()->Result<PathBuf, String>{
    let args = Cli::parse();
    let path = args.path;
    if !path.exists(){
        return Err("Invalid path".to_string())
    }
    if path.extension().unwrap() !=  OsStr::new("toml"){
        return Err("Configuration file must be in toml format".to_string())
    }
    Ok(path)
}


fn start_services(toml_str: String) -> Vec<ChildAndConfig>{
    let services: Tasks = toml::from_str(&toml_str).unwrap();
    let mut servicevec = Vec::<ChildAndConfig>::new();

    for i in 0..services.task.len(){
        let service = ChildAndConfig{
            child: new_process(services.task[i].path.clone()),
            config: services.task[i].clone(),
            prev_exit_status: None,
        };
        printerror(&service, &MajorEvents::Started);
        servicevec.push(service);
    }
    servicevec
}

// path points to the executable that will be spawned
fn new_process(path: String)->Child{
    Command::new(path).stdout(Stdio::piped()).spawn().unwrap()
}


fn printerror(service: &ChildAndConfig, event: &MajorEvents){
    let datetime = format!("{}", chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S"));
    let name = service.config.name.clone();
    let pid = service.child.id();
    
    let description = match &event{
        MajorEvents::OkStopped  => String::from("Process stopped with no errors"),
        MajorEvents::ErrStopped => String::from("Process stopped with errors"),
        MajorEvents::Started    => String::from("Process started"),
        MajorEvents::Restarted  => String::from("Process restarted"),
    };

    let event_type = match &event{
        MajorEvents::OkStopped  => String::from("Stopped-Ok"),
        MajorEvents::ErrStopped => String::from("Stopped-Err"),
        MajorEvents::Started    => String::from("Started"),
        MajorEvents::Restarted  => String::from("Restarted"),
    };

    let errormsg = format!("[{}] {} ({} ({})): {}", datetime, event_type, name, pid, description);
    eprintln!("{}", errormsg);
}


fn main() {

    
    let path: PathBuf;
    match get_path(){
        Ok(_path) => path = _path,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    }

    let toml_contents = std::fs::read_to_string(path).unwrap();
    let mut servicevec = start_services(toml_contents);


    loop {
        for service in &mut servicevec{
            let exit_status = service.child.try_wait().unwrap();

            // if child is exited now but it wasnt last iteration, the main logic will be executed
            if exit_status.is_some() && service.prev_exit_status.is_none() {
                let stopstatus = match &exit_status.unwrap().code(){
                    Some(_) => MajorEvents::OkStopped,
                    None       => MajorEvents::ErrStopped,
                };

                printerror(&service, &stopstatus);

                match &service.config.restart_condition{
                    RestartCondition::Never => {},
                    RestartCondition::OnError => {
                        if stopstatus == MajorEvents::ErrStopped{
                            service.child = new_process(service.config.path.clone());
                            printerror(&service, &MajorEvents::Restarted);
                        }

                    },
                    RestartCondition::Always => {
                            service.child = new_process(service.config.path.clone());
                            printerror(&service, &MajorEvents::Restarted);
                    },
                }
            }
            service.prev_exit_status = exit_status;
        }
    }
}

