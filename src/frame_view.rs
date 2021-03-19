use crate::frame_viewer::FrameViewer;
use crate::profile_data::ProfileData;
use crate::task_tree::TaskTree;
use gtk4::gio::File;
use gtk4::{Box as GtkBox, BoxExt, Orientation, WidgetExt};
use std::error::Error;

pub struct FrameView {
    widget: GtkBox,
    frame_viewer: FrameViewer,
    task_tree: TaskTree,
}

impl FrameView {
    pub fn new() -> Self {
        let frame_viewer = FrameViewer::new();
        let task_tree = TaskTree::new();
        task_tree.widget_ref().set_vexpand(true);

        let widget = GtkBox::new(Orientation::Vertical, 18);
        widget.append(frame_viewer.widget_ref());
        widget.append(task_tree.widget_ref());

        Self {
            widget,
            frame_viewer,
            task_tree,
        }
    }

    pub fn load_profile(&self, file: File) -> Result<(), Box<dyn Error>> {
        let profile = ProfileData::from_file(file)?;
        self.frame_viewer.load_profile(&profile);
        self.task_tree.load_profile(&profile);
        Ok(())
    }

    pub fn widget(&self) -> GtkBox {
        self.widget.clone()
    }

    pub fn widget_ref(&self) -> &GtkBox {
        &self.widget
    }
}
