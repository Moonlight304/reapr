use gtk4::{
    Align, Box, Button, Entry, Orientation, Window,
    prelude::{BoxExt, ButtonExt, GtkWindowExt, *},
};
use std::rc::Rc;

pub struct AddProcess {
    container: Box,
}

impl AddProcess {
    fn open_dialog(parent: &Window, on_submit: Rc<dyn Fn(String) -> Result<(), String>>) {
        let new_process_input = Entry::builder()
            .max_length(255)
            .placeholder_text("Enter a service name")
            .visibility(true)
            .build();

        let error_label = gtk4::Label::builder()
            .xalign(0.0)
            .visible(false)
            .build();
        error_label.add_css_class("error");

        let dialog_box = Box::builder()
            .spacing(10)
            .orientation(Orientation::Vertical)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .build();

        dialog_box.append(&new_process_input);
        dialog_box.append(&error_label);

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
        let error_label_clone = error_label.clone();

        new_process_input.connect_activate(move |entry| {
            let text = entry.text().trim().to_string();

            match on_submit(text) {
                Ok(_) => dialog_clone.close(),
                Err(err) => {
                    error_label_clone.set_label(&err);
                    error_label_clone.set_visible(true);
                }
            }
        });

        dialog.present();
    }

    pub fn new(on_submit: Rc<dyn Fn(String) -> Result<(), String>>) -> Self {
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

        let on_submit = on_submit.clone();
        add_process_button.connect_clicked(move |btn| {
            if let Some(parent_window) = btn.root().and_downcast::<Window>() {
                Self::open_dialog(&parent_window, on_submit.clone());
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
