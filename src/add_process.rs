use gtk4::{Box, Button, prelude::{BoxExt}};

pub struct AddProcess {
    container: Box,
}

impl AddProcess {
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
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(20)
        .halign(gtk4::Align::Center)
        .build();

        container.append(&add_process_button);

        Self { container }
    }

    pub fn widget(&self) -> &Box {
        &self.container
    }
}
