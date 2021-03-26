use crate::frame_view::FrameView;
use crate::profile_data::{ProfileData, TaskData, TaskObject};
use crate::task_view::TaskView;
use gtk4::gio::File;
use gtk4::{Align, Box as GtkBox, BoxExt, Label, ShortcutLabel, Stack, WidgetExt};
use libadwaita::StatusPage;
use std::error::Error;
use std::time::Duration;

pub struct Views {
    widget: Stack,
    views: Stack,
    frame_view: FrameView,
    task_view: TaskView,
}

impl Views {
    pub fn new() -> Self {
        let frame_view = FrameView::new();
        let task_view = TaskView::new();

        let views = Stack::new();
        views.add_titled(frame_view.widget(), Some("frame_view"), "Frame View");
        views.add_titled(task_view.widget(), Some("task_view"), "Task View");
        views
            .get_page(frame_view.widget())
            .unwrap()
            .set_icon_name("frame-view-symbolic");
        views
            .get_page(task_view.widget())
            .unwrap()
            .set_icon_name("task-view-symbolic");
        views.set_margin_top(18);
        views.set_margin_bottom(18);
        views.set_margin_start(18);
        views.set_margin_end(18);

        let status_page = StatusPage::new();
        status_page.set_icon_name(Some("profile-symbolic"));
        status_page.set_title(Some("Open a Profile"));
        let description_label = Label::new(Some("Press the Open profile button or press"));
        let description_shortcut = ShortcutLabel::new("<ctl>O");
        let description_box = GtkBox::new(gtk4::Orientation::Horizontal, 6);
        description_box.set_halign(Align::Center);
        description_box.append(&description_label);
        description_box.append(&description_shortcut);
        status_page.set_child(Some(&description_box));

        let widget = Stack::new();
        widget.add_child(&status_page);
        widget.add_named(&views, Some("views"));

        Self {
            widget,
            views,
            frame_view,
            task_view,
        }
    }

    pub fn load_profile(&self, file: File) -> Result<&Stack, Box<dyn Error>> {
        let tasks = ProfileData::from_file(file)?
            .frames
            .into_iter()
            .enumerate()
            .map(|(i, frame)| {
                TaskObject::new(TaskData {
                    name: format!("Frame #{}", i + 1),
                    duration: frame.duration,
                    subtasks: frame.tasks.clone(),
                })
            })
            .collect::<Vec<TaskObject>>();
        let above_threshold_count = tasks
            .iter()
            .filter(|frame| frame.get().duration > Duration::from_nanos(16666670))
            .count();

        self.frame_view.load_frames(&tasks, above_threshold_count);
        self.task_view.load_tasks(&tasks);

        self.widget.set_visible_child_name("views");

        Ok(&self.views)
    }

    pub fn widget(&self) -> &Stack {
        &self.widget
    }
}
