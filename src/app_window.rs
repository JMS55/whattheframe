use crate::views::Views;
use gtk4::prelude::ApplicationExt;
use gtk4::{
    Application, Box as GtkBox, BoxExt, Button, ButtonExt, CallbackAction, FileChooserAction,
    FileChooserExt, FileChooserNative, FileFilter, GtkWindowExt, InfoBar, Label, MessageType,
    NativeDialogExt, ResponseType, Shortcut, ShortcutController, ShortcutTrigger, WidgetExt,
};
use libadwaita::{ApplicationWindow, ApplicationWindowExt, HeaderBar, ViewSwitcher};

pub struct AppWindow {}

impl AppWindow {
    pub fn new(application: &Application) {
        let load_profile_error_label = Label::new(Some("Failed to Load Profile"));
        let load_profile_error_bar = InfoBar::new();
        load_profile_error_bar.add_child(&load_profile_error_label);
        load_profile_error_bar.set_message_type(MessageType::Error);
        load_profile_error_bar.set_show_close_button(true);
        load_profile_error_bar.connect_response(|bar, _| bar.hide());
        load_profile_error_bar.hide();

        let views = Views::new();

        let content_area = GtkBox::new(gtk4::Orientation::Vertical, 0);
        content_area.append(&load_profile_error_bar);
        content_area.append(views.widget());

        let open_profile_button = Button::with_label("Open Profile");

        let view_switcher = ViewSwitcher::new();

        let header_bar = HeaderBar::new();
        header_bar.pack_start(&open_profile_button);
        header_bar.set_title_widget(Some(&view_switcher));

        let window_content = GtkBox::new(gtk4::Orientation::Vertical, 0);
        window_content.append(&header_bar);
        window_content.append(&content_area);

        let shortcut_controller = ShortcutController::new();

        let window = ApplicationWindow::new(application);
        window.set_default_size(830, 560);
        ApplicationWindowExt::set_child(&window, Some(&window_content));
        window.add_controller(&shortcut_controller);

        let file_chooser = FileChooserNative::new(
            Some("Open Profile"),
            Some(&window),
            FileChooserAction::Open,
            None,
            None,
        );

        let wtf_filter = FileFilter::new();
        wtf_filter.set_name(Some("WhatTheFrame Profile (.wtf)"));
        wtf_filter.add_pattern("*.wtf");
        file_chooser.add_filter(&wtf_filter);
        let any_filter = FileFilter::new();
        any_filter.set_name(Some("Any"));
        any_filter.add_pattern("*");
        file_chooser.add_filter(&any_filter);

        open_profile_button.connect_clicked({
            let file_chooser = file_chooser.clone();
            move |_| file_chooser.show()
        });

        let open_profile_shorcut = Shortcut::new(
            Some(&ShortcutTrigger::parse_string("<Control>O").unwrap()),
            Some(&CallbackAction::new(Some(Box::new({
                let file_chooser = file_chooser.clone();
                move |_, _| {
                    file_chooser.show();
                    true
                }
            })))),
        );
        shortcut_controller.add_shortcut(&open_profile_shorcut);

        file_chooser.connect_response(move |file_chooser, response| {
            if response == ResponseType::Accept {
                if let Some(profile) = file_chooser.get_file() {
                    match views.load_profile(profile) {
                        Ok(views) => view_switcher.set_stack(Some(views)),
                        Err(_) => load_profile_error_bar.show(),
                    }
                }
            }
        });

        application.connect_activate(move |_| {
            window.show();
        });
    }
}
