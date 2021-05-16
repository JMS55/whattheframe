use crate::task_object::TaskObject;
use gtk4::glib::{self, Object};
use gtk4::prelude::{BoxExt, OrientableExt, WidgetExt};
use gtk4::subclass::prelude::{
    BoxImpl, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, OrientableImpl,
    WidgetImpl,
};
use gtk4::{Box as GtkBox, Label, Orientable, Orientation, Widget};

mod inner {
    use super::*;

    pub struct Task {
        pub name_label: Label,
        pub duration_label: Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Task {
        const NAME: &'static str = "Task";
        type Type = super::Task;
        type ParentType = GtkBox;
        type Interfaces = (Orientable,);

        fn new() -> Self {
            let name_label = Label::new(None);
            let duration_label = Label::new(None);
            duration_label.set_yalign(1.0);
            duration_label.add_css_class("caption-heading");
            duration_label.add_css_class("dim-label");

            Self {
                name_label,
                duration_label,
            }
        }
    }

    impl ObjectImpl for Task {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.set_orientation(Orientation::Horizontal);
            obj.set_spacing(12);
            obj.append(&self.name_label);
            obj.append(&self.duration_label);
        }
    }

    impl WidgetImpl for Task {}

    impl BoxImpl for Task {}

    impl OrientableImpl for Task {}
}

glib::wrapper! {
    pub struct Task(ObjectSubclass<inner::Task>) @extends GtkBox, Widget, @implements Orientable;
}

impl Task {
    pub fn new() -> Self {
        Object::new(&[]).unwrap()
    }

    pub fn set_task(&self, task: Option<&TaskObject>) {
        let this = inner::Task::from_instance(self);
        match task.map(|t| t.get()) {
            Some(task) => {
                this.name_label.set_label(&task.name);

                let task_duration_ms = task.duration.as_secs_f64() * 1000.0;
                let task_duation_label = format!("{:.2}ms", task_duration_ms);
                this.duration_label.set_label(&task_duation_label);
            }
            None => {
                this.name_label.set_label("");
                this.duration_label.set_label("");
            }
        }
    }
}
