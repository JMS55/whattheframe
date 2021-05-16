use crate::task_object::TaskObject;
use gtk4::cairo::Context;
use gtk4::glib::{self, Object};
use gtk4::prelude::DrawingAreaExt;
use gtk4::subclass::prelude::{
    DrawingAreaImpl, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, WidgetImpl,
};
use gtk4::{DrawingArea, Widget};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wtf::TaskData;

pub const FRAME_WIDTH: i32 = 12;
pub const FRAME_HEIGHT: i32 = 140;

mod inner {
    use super::*;

    pub struct Frame {
        pub data: Rc<RefCell<TaskObject>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Frame {
        const NAME: &'static str = "Frame";
        type Type = super::Frame;
        type ParentType = DrawingArea;

        fn new() -> Self {
            Self {
                data: Rc::new(RefCell::new(TaskObject::new(TaskData {
                    name: Box::from(""),
                    duration: Duration::default(),
                    subtasks: Box::new([]),
                }))),
            }
        }
    }

    impl ObjectImpl for Frame {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_content_width(FRAME_WIDTH);
            obj.set_content_height(FRAME_HEIGHT);

            obj.set_draw_func({
                let data = self.data.clone();
                move |_: &DrawingArea, canvas: &Context, _: i32, _: i32| {
                    let duration = data.borrow().get().duration;
                    let duration_ms = duration.as_secs_f64() * 1000.0;
                    let height = (duration_ms / 24.0).clamp(0.05, 1.0) * (FRAME_HEIGHT as f64);

                    canvas.rectangle(
                        1.0,
                        FRAME_HEIGHT as f64 - height,
                        FRAME_WIDTH as f64,
                        height,
                    );
                    if duration > Duration::from_nanos(16666670) {
                        canvas.set_source_rgb(237.0 / 255.0, 51.0 / 255.0, 59.0 / 255.0);
                    } else {
                        canvas.set_source_rgb(98.0 / 255.0, 160.0 / 255.0, 234.0 / 255.0);
                    }
                    canvas.fill().unwrap();
                }
            });
        }
    }

    impl WidgetImpl for Frame {}

    impl DrawingAreaImpl for Frame {}
}

glib::wrapper! {
    pub struct Frame(ObjectSubclass<inner::Frame>) @extends DrawingArea, Widget;
}

impl Frame {
    pub fn new() -> Self {
        Object::new(&[]).unwrap()
    }

    pub fn set_data(&self, data: TaskObject) {
        *inner::Frame::from_instance(self).data.borrow_mut() = data;
    }
}
