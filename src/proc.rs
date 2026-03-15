use dotenv::dotenv;
use gtk4::prelude::BoxExt;
use gtk4::{Box, Label};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;



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
            .label(format!("Name: {}, Status: {}", process.name, process.status))
            .build();

            process_list_container.append(&label);
        }

        return process_list_container;
    }

    pub fn get_all_processes(&self) -> Vec<Process> {
        self.processes.clone()
    }
}
