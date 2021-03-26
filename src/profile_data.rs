use gtk4::gio::{File, FileExt, NONE_CANCELLABLE};
use gtk4::glib::{self, Object};
use gtk4::subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt};
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefCell};
use std::error::Error;
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub frames: Box<[FrameData]>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FrameData {
    pub duration: Duration,
    pub tasks: Box<[TaskData]>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TaskData {
    pub name: String,
    pub duration: Duration,
    pub subtasks: Box<[TaskData]>,
}

impl ProfileData {
    pub fn from_file(file: File) -> Result<Self, Box<dyn Error>> {
        let (bytes, _) = file.load_contents(NONE_CANCELLABLE)?;
        let profile = bincode::deserialize(&bytes)?;
        Ok(profile)
    }
}

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
                name: String::new(),
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
