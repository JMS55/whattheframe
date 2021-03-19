mod app_window;
mod frame;
mod frame_view;
mod frame_viewer;
mod profile_data;
mod task_tree;
mod task_view;

use crate::app_window::AppWindow;
use gtk4::gio::ApplicationFlags;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::Application;
use std::env;

fn main() {
    let application = Application::new(
        Some("com.github.jms55.WhatTheFrame"),
        ApplicationFlags::default(),
    )
    .expect("GTK Application::new() failed");
    application.connect_activate(|app| AppWindow::new(app));
    application.run(&env::args().collect::<Vec<_>>());
}
