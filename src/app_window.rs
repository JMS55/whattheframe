use crate::frame_view::FrameView;
use crate::task_view::TaskView;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, BoxExt, Button, ButtonExt, FileChooserAction,
    FileChooserExt, FileChooserNative, FileFilter, GtkWindowExt, HeaderBar, InfoBar, Label,
    MessageType, NativeDialogExt, ResponseType, Stack, StackSwitcher, WidgetExt,
};

pub struct AppWindow {}

impl AppWindow {
    pub fn new(application: &Application) {
        let frame_view = FrameView::new();

        let views = Stack::new();
        views.add_titled(frame_view.widget(), Some("frame_view"), "Frame View");
        views.add_titled(TaskView::new().widget(), Some("task_view"), "Task View");
        views.set_margin_top(18);
        views.set_margin_bottom(18);
        views.set_margin_start(18);
        views.set_margin_end(18);

        let load_profile_error_label = Label::new(Some("Failed to Load Profile"));
        let load_profile_error_bar = InfoBar::new();
        load_profile_error_bar.add_child(&load_profile_error_label);
        load_profile_error_bar.set_message_type(MessageType::Error);
        load_profile_error_bar.set_show_close_button(true);
        load_profile_error_bar.connect_response(|bar, _| bar.hide());
        load_profile_error_bar.hide();

        let content_area = GtkBox::new(gtk4::Orientation::Vertical, 0);
        content_area.append(&load_profile_error_bar);
        content_area.append(&views);

        let open_profile_button = Button::from_icon_name(Some("document-open-symbolic"));

        let view_switcher = StackSwitcher::new();
        view_switcher.set_stack(Some(&views));

        let header_bar = HeaderBar::new();
        header_bar.pack_start(&open_profile_button);
        header_bar.set_title_widget(Some(&view_switcher));

        let window = ApplicationWindow::new(application);
        window.set_default_size(830, 560);
        window.set_titlebar(Some(&header_bar));
        window.set_child(Some(&content_area));

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
        frame_view
            .frame_timeline_placeholder_widget()
            .connect_clicked({
                let file_chooser = file_chooser.clone();
                move |_| file_chooser.show()
            });

        file_chooser.connect_response(move |file_chooser, response| {
            if response == ResponseType::Accept {
                if let Some(profile) = file_chooser.get_file() {
                    if let Err(_) = frame_view.load_profile(profile) {
                        load_profile_error_bar.show();
                    }
                }
            }
        });

        window.show();
    }
}
