use dotenv::dotenv;
use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, Label, MenuButton, Orientation, Popover, Separator, Switch,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Process {
    name: String,
}

#[derive(Clone, Copy)]
enum ServiceAction {
    Start,
    Stop,
    Enable,
    Disable,
}

impl ServiceAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Enable => "enable",
            Self::Disable => "disable",
        }
    }
}

#[derive(Debug, Clone)]
struct ServiceState {
    active: bool,
    active_label: String,
    enabled_label: String,
}

pub struct ProcessManager;

impl ProcessManager {
    pub fn new() -> Self {
        dotenv().ok();
        Self
    }

    fn process_file_path() -> Result<PathBuf, String> {
        dotenv().ok();
        let path = std::env::var("PROC_LIST_PATH")
            .map_err(|_| "Missing PROC_LIST_PATH in .env".to_string())?;
        Ok(PathBuf::from(path))
    }

    fn normalize_name(name: &str) -> String {
        let trimmed = name.trim();
        if trimmed.contains('.') {
            trimmed.to_string()
        } else {
            format!("{}.service", trimmed)
        }
    }

    fn read_processes_result() -> Result<Vec<Process>, String> {
        let path = Self::process_file_path()?;

        if !path.exists() {
            fs::write(&path, "[]").map_err(|e| format!("Failed to initialize process file: {e}"))?;
        }

        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read process file {}: {e}", path.display()))?;

        if data.trim().is_empty() {
            return Ok(Vec::new());
        }

        serde_json::from_str::<Vec<Process>>(&data)
            .map_err(|e| format!("Failed to parse process JSON {}: {e}", path.display()))
    }

    fn write_processes(processes: &[Process]) -> Result<(), String> {
        let path = Self::process_file_path()?;
        let payload = serde_json::to_string_pretty(processes)
            .map_err(|e| format!("Failed to serialize process list: {e}"))?;

        fs::write(&path, payload)
            .map_err(|e| format!("Failed to write process file {}: {e}", path.display()))
    }

    pub fn get_all_processes() -> Vec<Process> {
        match Self::read_processes_result() {
            Ok(list) => list,
            Err(err) => {
                eprintln!("{err}");
                Vec::new()
            }
        }
    }

    pub fn new_process(process_name: String) -> Result<(), String> {
        let normalized = Self::normalize_name(&process_name);
        if normalized.is_empty() || normalized == ".service" {
            return Err("Service name cannot be empty".to_string());
        }

        if !Self::service_exists(&normalized)? {
            return Err(format!(
                "{} does not exist in systemd (unit not found)",
                normalized
            ));
        }

        let mut all_processes = Self::read_processes_result()?;

        let already_exists = all_processes
            .iter()
            .any(|p| p.name.eq_ignore_ascii_case(&normalized));

        if already_exists {
            return Err(format!("{} is already in your managed list", normalized));
        }

        all_processes.push(Process { name: normalized });
        all_processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Self::write_processes(&all_processes)
    }

    pub fn remove_process(process_name: &str) -> Result<(), String> {
        let mut all_processes = Self::read_processes_result()?;
        let before = all_processes.len();

        all_processes.retain(|p| p.name != process_name);

        if before == all_processes.len() {
            return Err(format!("{} was not found in your managed list", process_name));
        }

        Self::write_processes(&all_processes)
    }

    fn systemctl_state(command: &str, process_name: &str) -> Result<String, String> {
        let output = Command::new("systemctl")
            .arg(command)
            .arg(process_name)
            .output()
            .map_err(|e| format!("Failed to run systemctl {command} for {process_name}: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !stdout.is_empty() {
            Ok(stdout)
        } else if !stderr.is_empty() {
            Ok(stderr)
        } else if output.status.success() {
            Ok("ok".to_string())
        } else {
            Ok("unknown".to_string())
        }
    }

    fn service_exists(process_name: &str) -> Result<bool, String> {
        let output = Command::new("systemctl")
            .arg("show")
            .arg(process_name)
            .arg("--property")
            .arg("LoadState")
            .arg("--value")
            .output()
            .map_err(|e| format!("Failed to verify service {process_name}: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            if stderr.contains("could not be found") || stderr.contains("not-found") {
                return Ok(false);
            }

            return Err(format!(
                "Failed to verify service {}: {}",
                process_name,
                if stderr.is_empty() { "unknown error" } else { &stderr }
            ));
        }

        Ok(!stdout.eq_ignore_ascii_case("not-found"))
    }

    fn get_service_state(process_name: &str) -> ServiceState {
        let active_label =
            Self::systemctl_state("is-active", process_name).unwrap_or_else(|_| "unknown".to_string());
        let enabled_label =
            Self::systemctl_state("is-enabled", process_name).unwrap_or_else(|_| "unknown".to_string());

        let active = active_label == "active";

        ServiceState {
            active,
            active_label,
            enabled_label,
        }
    }

    fn run_systemctl_action(process_name: &str, action: ServiceAction) -> Result<(), String> {
        let output = Command::new("systemctl")
            .arg(action.as_str())
            .arg(process_name)
            .output()
            .map_err(|e| {
                format!(
                    "Failed to run systemctl {} for {}: {}",
                    action.as_str(),
                    process_name,
                    e
                )
            })?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!(
                "systemctl {} {} failed with exit code {}",
                action.as_str(),
                process_name,
                output.status
            ))
        } else {
            Err(stderr)
        }
    }

    pub fn render_processes(&self) -> Box {
        let process_list_container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_start(20)
            .margin_end(20)
            .margin_bottom(20)
            .build();

        process_list_container.add_css_class("process-list");

        Self::refresh_processes(&process_list_container);

        process_list_container
    }

    fn build_process_row(process_name: &str, container: &Box) -> Box {
        let state = Self::get_service_state(process_name);

        let row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(10)
            .margin_bottom(10)
            .margin_start(12)
            .margin_end(12)
            .halign(Align::Fill)
            .build();
        row.add_css_class("service-card");

        let left_col = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .hexpand(true)
            .halign(Align::Start)
            .build();

        let name_line = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .halign(Align::Start)
            .build();

        let state_dot = Label::builder().label("●").build();
        state_dot.add_css_class("state-dot");
        if state.active {
            state_dot.add_css_class("state-dot-active");
        } else {
            state_dot.add_css_class("state-dot-inactive");
        }

        let name_label = Label::builder().label(process_name).xalign(0.0).build();
        name_label.add_css_class("service-name");

        let status_line = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .halign(Align::Start)
            .build();

        let meta_label = Label::builder()
            .label(format!(
                "Status: {} | Boot: {}",
                state.active_label, state.enabled_label
            ))
            .xalign(0.0)
            .build();
        meta_label.add_css_class("service-meta");

        name_line.append(&state_dot);
        name_line.append(&name_label);

        status_line.append(&meta_label);

        left_col.append(&name_line);
        left_col.append(&status_line);

        let active_switch = Switch::builder()
            .active(state.active)
            .valign(Align::Center)
            .build();

        let switch_process_name = process_name.to_string();
        let switch_container = container.clone();
        active_switch.connect_active_notify(move |sw| {
            let result = if sw.is_active() {
                ProcessManager::run_systemctl_action(&switch_process_name, ServiceAction::Start)
            } else {
                ProcessManager::run_systemctl_action(&switch_process_name, ServiceAction::Stop)
            };

            if let Err(err) = result {
                eprintln!("{err}");
            }

            ProcessManager::refresh_processes(&switch_container);
        });

        let popover_content = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        let enable_btn = Button::with_label("Enable on boot");
        let disable_btn = Button::with_label("Disable on boot");
        let remove_btn = Button::with_label("Remove from list");
        remove_btn.add_css_class("danger-btn");

        let enable_name = process_name.to_string();
        let enable_container = container.clone();
        enable_btn.connect_clicked(move |_| {
            if let Err(err) = ProcessManager::run_systemctl_action(&enable_name, ServiceAction::Enable) {
                eprintln!("{err}");
            }
            ProcessManager::refresh_processes(&enable_container);
        });

        let disable_name = process_name.to_string();
        let disable_container = container.clone();
        disable_btn.connect_clicked(move |_| {
            if let Err(err) = ProcessManager::run_systemctl_action(&disable_name, ServiceAction::Disable) {
                eprintln!("{err}");
            }
            ProcessManager::refresh_processes(&disable_container);
        });

        let remove_name = process_name.to_string();
        let remove_container = container.clone();
        remove_btn.connect_clicked(move |_| {
            if let Err(err) = ProcessManager::remove_process(&remove_name) {
                eprintln!("{err}");
            }
            ProcessManager::refresh_processes(&remove_container);
        });

        popover_content.append(&enable_btn);
        popover_content.append(&disable_btn);
        popover_content.append(&Separator::new(Orientation::Horizontal));
        popover_content.append(&remove_btn);

        let popover = Popover::builder().child(&popover_content).build();

        let gear_menu = MenuButton::builder()
            .icon_name("emblem-system-symbolic")
            .popover(&popover)
            .valign(Align::Center)
            .build();

        row.append(&left_col);
        row.append(&active_switch);
        row.append(&gear_menu);

        row
    }

    pub fn refresh_processes(container: &Box) {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let processes = Self::get_all_processes();

        if processes.is_empty() {
            let empty_label = Label::builder()
                .label("No services yet. Add one above to start managing it.")
                .xalign(0.0)
                .build();
            empty_label.add_css_class("service-meta");
            container.append(&empty_label);
            return;
        }

        for process in processes {
            let row = Self::build_process_row(&process.name, container);
            container.append(&row);
        }
    }

    pub fn install_css() {
        let css = r#"
            .process-list {
                padding: 2px;
            }

            .service-card {
                border-radius: 10px;
                border: 1px solid #e7e7e766;
                padding: 10px 12px;
            }

            .service-name {
                font-size: 16px;
                font-weight: 700;
            }

            .service-meta {
                font-size: 12px;
            }

            .state-dot {
                font-size: 13px;
                margin-top: -1px;
            }

            .state-dot-active {
                color: #1f9d42;
            }

            .state-dot-inactive {
                color: #c7392f;
            }

            .danger-btn {
                color: #9f1d1d;
            }
        "#;

        let provider = gtk4::CssProvider::new();
        provider.load_from_data(css);

        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }
}
