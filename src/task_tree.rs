use crate::profile_data::ProfileData;
use gtk4::Button;

pub struct TaskTree {
    widget: Button,
}

impl TaskTree {
    pub fn new() -> Self {
        let widget = Button::with_label("TODO");

        Self { widget }
    }

    pub fn load_profile(&self, profile: &ProfileData) {
        // TODO
    }

    pub fn widget(&self) -> Button {
        self.widget.clone()
    }

    pub fn widget_ref(&self) -> &Button {
        &self.widget
    }
}
