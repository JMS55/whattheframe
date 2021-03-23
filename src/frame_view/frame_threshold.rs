use gtk4::cairo::{Context, LineCap};
use gtk4::{Box as GtkBox, BoxExt, DrawingArea, DrawingAreaExt, Label, Orientation, WidgetExt};

pub struct FrameThreshold {
    widget: GtkBox,
}

impl FrameThreshold {
    pub fn new() -> Self {
        let label = Label::new(Some("16.6ms"));
        label.add_css_class("caption-heading");

        let drawing_area = DrawingArea::new();
        drawing_area.set_draw_func(
            |_: &DrawingArea, canvas: &Context, width: i32, height: i32| {
                let y = height as f64 / 2.0;
                canvas.move_to(0.0, y);
                canvas.line_to(width as f64, y);
                canvas.set_source_rgba(51.0 / 255.0, 209.0 / 255.0, 122.0 / 255.0, 0.7);
                canvas.set_line_width(3.0);
                canvas.set_dash(&[8.0], 0.0);
                canvas.set_line_cap(LineCap::Round);
                canvas.stroke();
            },
        );
        drawing_area.set_hexpand(true);

        let widget = GtkBox::new(Orientation::Horizontal, 6);
        widget.append(&label);
        widget.append(&drawing_area);

        Self { widget }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }
}
