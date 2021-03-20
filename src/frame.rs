use crate::profile_data::FrameDataObject;
use gtk4::cairo::Context;
use gtk4::glib::{self, Object};
use gtk4::subclass::prelude::{
    DrawingAreaImpl, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, WidgetImpl,
};
use gtk4::{DrawingArea, DrawingAreaExt, Widget};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

mod inner {
    use super::*;

    pub struct Frame {
        pub data: Rc<RefCell<Option<FrameDataObject>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Frame {
        const NAME: &'static str = "Frame";
        type Type = super::Frame;
        type ParentType = DrawingArea;

        fn new() -> Self {
            Self {
                data: Rc::new(RefCell::new(None)),
            }
        }
    }

    impl ObjectImpl for Frame {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_content_width(14);
            obj.set_content_height(162);

            obj.set_draw_func({
                let data = self.data.clone();
                move |_: &DrawingArea, canvas: &Context, _: i32, _: i32| {
                    if let Some(data) = &*data.borrow() {
                        let duration = data.get().duration;

                        let duration_ms = duration.as_secs_f64() * 1000.0;
                        let height = (duration_ms / 24.0).clamp(0.1, 1.0) * 160.0;
                        canvas.rectangle(1.0, 161.0 - height, 12.0, height);

                        if duration > Duration::from_nanos(16666670) {
                            canvas.set_source_rgb(246.0 / 255.0, 97.0 / 255.0, 81.0 / 255.0);
                        } else {
                            canvas.set_source_rgb(153.0 / 255.0, 193.0 / 255.0, 241.0 / 255.0);
                        }
                        canvas.fill_preserve();

                        if duration > Duration::from_nanos(16666670) {
                            canvas.set_source_rgb(224.0 / 255.0, 27.0 / 255.0, 36.0 / 255.0);
                        } else {
                            canvas.set_source_rgb(98.0 / 255.0, 160.0 / 255.0, 234.0 / 255.0);
                        }
                        canvas.set_line_width(1.0);
                        canvas.stroke();
                    }
                }
            });
        }
    }

    impl WidgetImpl for Frame {}

    impl DrawingAreaImpl for Frame {}
}

glib::wrapper! {
    pub struct Frame(ObjectSubclass<inner::Frame>) @extends Widget, DrawingArea;
}

impl Frame {
    pub fn new() -> Self {
        Object::new(&[]).unwrap()
    }

    pub fn set_data(&self, data: Option<FrameDataObject>) {
        *inner::Frame::from_instance(self).data.borrow_mut() = data;
    }
}
