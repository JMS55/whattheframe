use gtk4::Button;

pub struct TaskView {
    widget: Button,
}

impl TaskView {
    pub fn new() -> Self {
        let widget = Button::with_label("TODO");

        Self { widget }
    }

    pub fn widget(&self) -> &Button {
        &self.widget
    }
}
