use gtk4::*;
use gtk4::{ApplicationWindow, prelude::*};
use gtk4::{Application, glib};

const APP_ID: &str = "com.gtk_rs.reapr";

fn main () -> glib::ExitCode {
    let app = Application::builder()
    .application_id(APP_ID)
    .build();

    app.connect_activate(activate);

    app.run()
}

fn activate(app: &Application) {

    let container = Box::new(Orientation::Vertical, 10);

    let window = ApplicationWindow::builder()
    .application(app)
    .title("Reapr")
    .default_height(600)
    .default_width(720)
    .child(&container)
    .build();

    window.present();
}