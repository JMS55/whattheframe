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

    pub struct FrameDataObject(pub RefCell<FrameData>);

    #[glib::object_subclass]
    impl ObjectSubclass for FrameDataObject {
        const NAME: &'static str = "FrameDataObject";
        type Type = super::FrameDataObject;
        type ParentType = Object;

        fn new() -> Self {
            Self(RefCell::new(FrameData {
                duration: Duration::from_secs(0),
                tasks: Box::new([]),
            }))
        }
    }

    impl ObjectImpl for FrameDataObject {}
}

glib::wrapper! {
    pub struct FrameDataObject(ObjectSubclass<inner::FrameDataObject>);
}

impl FrameDataObject {
    pub fn new(data: FrameData) -> Self {
        let obj = Object::new(&[]).unwrap();
        *inner::FrameDataObject::from_instance(&obj).0.borrow_mut() = data;
        obj
    }

    pub fn get(&self) -> Ref<FrameData> {
        inner::FrameDataObject::from_instance(self).0.borrow()
    }
}
