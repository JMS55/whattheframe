use gtk4::gio::{File, FileExt, NONE_CANCELLABLE};
use gtk4::glib::{self, Object};
use gtk4::subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt};
use std::cell::{Ref, RefCell};
use std::error::Error;
use std::time::Duration;
use wtf::{ProfileData, TaskData};

mod inner {
    use super::*;

    pub struct TaskObject(pub RefCell<TaskData>);

    #[glib::object_subclass]
    impl ObjectSubclass for TaskObject {
        const NAME: &'static str = "TaskObject";
        type Type = super::TaskObject;
        type ParentType = Object;

        fn new() -> Self {
            Self(RefCell::new(TaskData {
                name: Box::from(""),
                duration: Duration::default(),
                subtasks: Box::new([]),
            }))
        }
    }

    impl ObjectImpl for TaskObject {}
}

glib::wrapper! {
    pub struct TaskObject(ObjectSubclass<inner::TaskObject>);
}

impl TaskObject {
    pub fn new(data: TaskData) -> Self {
        let obj = Object::new(&[]).unwrap();
        *inner::TaskObject::from_instance(&obj).0.borrow_mut() = data;
        obj
    }

    pub fn get(&self) -> Ref<TaskData> {
        inner::TaskObject::from_instance(self).0.borrow()
    }
}

pub fn profile_data_from_file(file: File) -> Result<ProfileData, Box<dyn Error>> {
    let (bytes, _) = file.load_contents(NONE_CANCELLABLE)?;
    wtf::profile_data_from_bytes(&bytes)
}
