use crate::frame_view::FrameTimeline;
use crate::frame_view::TaskTree;
use crate::profile_data::TaskObject;
use gtk4::{Box as GtkBox, BoxExt, Orientation, WidgetExt};
use std::rc::Rc;

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

    pub fn load_frames(&self, frames: &[TaskObject], above_threshold_count: usize) {
        let on_timeline_frame_selection_change = {
            let task_tree = self.task_tree.clone();
            move |frame| task_tree.set_frame(frame)
        };
        self.frame_timeline.load_frames(
            frames,
            above_threshold_count,
            on_timeline_frame_selection_change,
        );
    }

    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }
}
