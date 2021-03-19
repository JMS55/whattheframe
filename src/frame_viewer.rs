use crate::profile::ProfileData;
use gtk4::Button;

pub struct FrameViewer {
    widget: Button,
}

impl FrameViewer {
    pub fn new() -> Self {
        let widget = Button::with_label("TODO");

        Self { widget }
    }

    pub fn load_profile(&self, profile: &ProfileData) {
        todo!()
    }

    pub fn widget(&self) -> Button {
        self.widget.clone()
    }

    pub fn widget_ref(&self) -> &Button {
        &self.widget
    }
}
