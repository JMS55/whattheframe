use gtk4::gio::{File, FileExt, NONE_CANCELLABLE};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct ProfileData {
    pub frames: Box<[FrameData]>,
}

#[derive(Serialize, Deserialize)]
pub struct FrameData {
    pub duration: Duration,
    pub tasks: Box<[TaskData]>,
}

#[derive(Serialize, Deserialize)]
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
