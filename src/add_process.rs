use gtk4::{
    Align, Box, Button, Entry, Orientation, Window,
    prelude::{BoxExt, ButtonExt, GtkWindowExt, *},
};

use crate::proc::ProcessManager;

pub struct AddProcess {
    container: Box,
}

impl AddProcess {
    fn open_dialog<F>(parent: &Window, on_submit: F)
    where 
        F: Fn(String) + 'static,
    {
        let new_process_input = Entry::builder()
            .max_length(10)
            .placeholder_text("Enter a Process Name")
            .visibility(true)
            .build();

        let dialog_box = Box::builder()
            .spacing(10)
            .orientation(Orientation::Vertical)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .build();

        dialog_box.append(&new_process_input);

        let dialog = Window::builder()
            .title("Add New Process")
            .modal(true)
            .transient_for(parent)
            .default_width(600)
            .default_height(400)
            .destroy_with_parent(true)
            .child(&dialog_box)
            .build();

        let dialog_clone = dialog.clone();

        new_process_input.connect_activate(move |entry| {
            let text = entry.text().to_string();

            on_submit(text);

            dialog_clone.close();
        });

        dialog.present();
    }

    pub fn new() -> Self {
        let add_process_button = Button::builder()
            .label("Add Process")
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .width_request(400)
            .build();

        let container = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(20)
            .halign(Align::Center)
            .build();

        container.append(&add_process_button);

        add_process_button.connect_clicked(move |btn| {
            if let Some(parent_window) = btn.root().and_downcast::<Window>() {
                Self::open_dialog(&parent_window, |process_name| {
                    println!("{}", process_name);

                    let systemctl = systemctl::SystemCtl::default();

                    let process_status = systemctl.get_active_state(&process_name);

                    match process_status {
                        Ok(state) => {
                            println!("State: {}", state);

                            // let mut process_state;
                            // if state.to_string() == "Inactive"

                            ProcessManager::new_process(process_name, state.to_string());
                        },
                        Err(e) => println!("Error: {}", e),
                    }
                    
                });
            }
            else {
                eprintln!("Warning: Button is not attached to a Window yet.");
            }
        });

        Self { container }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
