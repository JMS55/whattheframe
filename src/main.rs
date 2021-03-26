mod app_window;
mod frame_view;
mod profile_data;
mod task_view;

use crate::app_window::AppWindow;
use gtk4::gdk::Display;
use gtk4::gio::{resources_register_include, ApplicationFlags};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::{Application, CssProvider, StyleContext, STYLE_PROVIDER_PRIORITY_APPLICATION};
use std::env;

fn main() {
    resources_register_include!("compiled.gresource").unwrap();

    let application = Application::new(
        Some("com.github.jms55.WhatTheFrame"),
        ApplicationFlags::default(),
    )
    .expect("GTK Application::new() failed");
    application.connect_startup(|app| {
        libadwaita::init();

        let css_provider = CssProvider::new();
        css_provider.load_from_resource("/com/github/jms55/WhatTheFrame/stylesheet.css");
        StyleContext::add_provider_for_display(
            &Display::get_default().unwrap(),
            &css_provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        AppWindow::new(app);
    });
    application.run(&env::args().collect::<Vec<_>>());
}
