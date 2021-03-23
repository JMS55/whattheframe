use crate::frame_view::FrameView;
use crate::task_view::TaskView;
use gtk4::{
    Application, ApplicationWindow, Button, ButtonExt, FileChooserAction, FileChooserExt,
    FileChooserNative, GtkWindowExt, HeaderBar, NativeDialogExt, ResponseType, Stack,
    StackSwitcher, WidgetExt,
};

pub struct AppWindow {}

impl AppWindow {
    pub fn new(application: &Application) {
        let window = ApplicationWindow::new(application);

        let frame_view = FrameView::new();

        let views = Stack::new();
        views.add_titled(frame_view.widget(), Some("frame_view"), "Frame View");
        views.add_titled(TaskView::new().widget(), Some("task_view"), "Task View");
        views.set_margin_top(18);
        views.set_margin_bottom(18);
        views.set_margin_start(18);
        views.set_margin_end(18);

        let open_profile_button = Button::from_icon_name(Some("document-open-symbolic"));

        // TODO: Only show .wtf files
        let file_chooser = FileChooserNative::new(
            Some("Open Profile"),
            Some(&window),
            FileChooserAction::Open,
            None,
            None,
        );
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
                    frame_view
                        .load_profile(profile)
                        .expect("TODO: Show an error popup");
                }
            }
        });

        let view_switcher = StackSwitcher::new();
        view_switcher.set_stack(Some(&views));

        let header_bar = HeaderBar::new();
        header_bar.pack_start(&open_profile_button);
        header_bar.set_title_widget(Some(&view_switcher));

        window.set_titlebar(Some(&header_bar));
        window.set_child(Some(&views));
        window.show();
    }
}
