mod add_process;
mod proc;

use gtk4::*;
use gtk4::{Application, glib};
use gtk4::{ApplicationWindow, prelude::*};
use std::rc::Rc;

const APP_ID: &str = "com.gtk_rs.reapr";
fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(activate);

    app.run()
}

fn activate(app: &Application) {
    proc::ProcessManager::install_css();

    let process_manager = proc::ProcessManager::new();

    let process_list_container = process_manager.render_processes();
    let process_list_container_for_refresh = process_list_container.clone();

    let on_submit = Rc::new(move |process_name: String| {
        proc::ProcessManager::new_process(process_name)?;
        proc::ProcessManager::refresh_processes(&process_list_container_for_refresh);
        Ok(())
    });

    let add_process_stuff = add_process::AddProcess::new(on_submit);

    let container = Box::new(Orientation::Vertical, 10);
    container.append(add_process_stuff.widget());
    container.append(&process_list_container);


    // render main application window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Reapr")
        .default_height(600)
        .default_width(720)
        .child(&container)
        .build();

    window.present();
}
