use crate::frame_view::FrameTimeline;
use crate::frame_view::TaskTree;
use crate::profile_data::{ProfileData, TaskData, TaskObject};
use gtk4::gio::File;
use gtk4::{Box as GtkBox, BoxExt, Button, Orientation, WidgetExt};
use std::error::Error;
use std::rc::Rc;
use std::time::Duration;

pub struct FrameView {
    widget: GtkBox,
    frame_timeline: FrameTimeline,
    task_tree: Rc<TaskTree>,
}

impl FrameView {
    pub fn new() -> Self {
        let frame_timeline = FrameTimeline::new();

        let task_tree = Rc::new(TaskTree::new());
        task_tree.widget().set_vexpand(true);

        let widget = GtkBox::new(Orientation::Vertical, 18);
        widget.append(frame_timeline.widget());
        widget.append(task_tree.widget());

        Self {
            widget,
            frame_timeline,
            task_tree,
        }
    }

    pub fn load_profile(&self, file: File) -> Result<(), Box<dyn Error>> {
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

        let on_timeline_frame_selection_change = {
            let task_tree = self.task_tree.clone();
            move |frame| task_tree.set_frame(frame)
        };
        self.frame_timeline.load_frames(
            &tasks,
            above_threshold_count,
            on_timeline_frame_selection_change,
        );
        Ok(())
    }

    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }

    pub fn frame_timeline_placeholder_widget(&self) -> &Button {
        self.frame_timeline.placeholder_widget()
    }
}
