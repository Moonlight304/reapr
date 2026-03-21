use dotenv::dotenv;
use gtk4::prelude::BoxExt;
use gtk4::{Box, Label};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::os::unix::process;
use std::path::Path;
use std::{env, fs};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Process {
    name: String,
    status: String,
}

pub struct ProcessManager {
    processes: Vec<Process>,
}

impl ProcessManager {
    pub fn new() -> Self {
        dotenv().ok();

        let proc_list_path = env::var("PROC_LIST_PATH").expect("Couldn't load env");

        let path = Path::new(&proc_list_path);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        let mut data = String::new();
        match file.read_to_string(&mut data) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => println!("{} contains:\n{}", display, data),
        }

        let initial_processes: Vec<Process> = match serde_json::from_str(&data) {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse JSON: {}", e),
        };

        Self {
            processes: initial_processes,
        }
    }

    pub fn render_processes(&self) -> Box {
        let process_list_container = Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(10)
            .build();

        for process in &self.processes {
            let label = Label::builder()
                .label(format!(
                    "Name: {}, Status: {}",
                    process.name, process.status
                ))
                .build();

            process_list_container.append(&label);
        }

        return process_list_container;
    }

    pub fn get_all_processes() -> Vec<Process> {
        dotenv().ok();

        let proc_list_path = env::var("PROC_LIST_PATH").expect("Couldn't load env");

        let path = Path::new(&proc_list_path);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        let mut data = String::new();
        match file.read_to_string(&mut data) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => println!("{} contains:\n{}", display, data),
        }

        let initial_processes: Vec<Process> = match serde_json::from_str(&data) {
            Ok(p) => p,
            Err(e) => panic!("Failed to parse JSON: {}", e),
        };

        initial_processes
    }

    pub fn new_process(process_name: String, process_status: String) {
        dotenv().ok();
        let proc_list_path = env::var("PROC_LIST_PATH").expect("Couldn't load env");

        let path = Path::new(&proc_list_path);

        let new_process_object = Process {
            name: process_name,
            status: process_status,
        };

        let mut all_processes = ProcessManager::get_all_processes();
        all_processes.push(new_process_object);

        let json_all_processes = serde_json::to_string_pretty(&all_processes)
            .expect("Failed to serialize the process list");

        fs::write(path, json_all_processes).expect("Failed to write to the JSON file");

        
    }
}
