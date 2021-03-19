use crate::profile_data::FrameData;
use gtk4::cairo::Context;
use gtk4::{DrawingArea, DrawingAreaExt};
use std::rc::Rc;
use std::time::Duration;

pub struct Frame {
    data: Rc<FrameData>,
    widget: DrawingArea,
}

impl Frame {
    pub fn new(data: FrameData) -> Self {
        let data = Rc::new(data);

        let widget = DrawingArea::new();
        widget.set_content_width(14);
        widget.set_content_height(202);
        widget.set_draw_func({
            let data = data.clone();
            move |_: &DrawingArea, canvas: &Context, _: i32, _: i32| {
                canvas.rectangle(1.0, 1.0, 12.0, 150.0);

                if data.duration > Duration::from_nanos(16666670) {
                    canvas.set_source_rgb(246.0 / 255.0, 97.0 / 255.0, 81.0 / 255.0);
                } else {
                    canvas.set_source_rgb(153.0 / 255.0, 193.0 / 255.0, 241.0 / 255.0);
                }
                canvas.fill_preserve();

                if data.duration > Duration::from_nanos(16666670) {
                    canvas.set_source_rgb(224.0 / 255.0, 27.0 / 255.0, 36.0 / 255.0);
                } else {
                    canvas.set_source_rgb(98.0 / 255.0, 160.0 / 255.0, 234.0 / 255.0);
                }
                canvas.set_line_width(1.0);
                canvas.stroke();
            }
        });

        Self { data, widget }
    }

    pub fn widget(&self) -> DrawingArea {
        self.widget.clone()
    }

    pub fn widget_ref(&self) -> &DrawingArea {
        &self.widget
    }
}
