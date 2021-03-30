use flume::Sender;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use snap::read::FrameDecoder;
use snap::write::FrameEncoder;
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Profiler {
    sender: Sender<ProfilerMessage>,
    thread: JoinHandle<()>,
}

static PROFILER: Lazy<RwLock<Option<Profiler>>> = Lazy::new(|| {
    let (sender, reciever) = flume::unbounded();

    let file_name = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let file = File::create(format!("{}.wtf", file_name))
        .expect("Error: Failed to create wtf::Profiler file");
    let mut file = FrameEncoder::new(file);

    let thread = thread::spawn(move || {
        for msg in reciever.iter() {
            match msg {
                ProfilerMessage::Frame { elapsed } => {
                    let frame = FrameData {
                        duration: elapsed,
                        tasks: todo!(),
                    };
                    bincode::serialize_into(&mut file, &frame).unwrap();
                }
                ProfilerMessage::TaskStart { name } => todo!(),
                ProfilerMessage::TaskEnd { name, elapsed } => todo!(),
            }
        }
    });

    RwLock::new(Some(Profiler { sender, thread }))
});

impl Profiler {
    pub fn new_frame() -> FrameProfile {
        FrameProfile::new()
    }

    pub fn profile_task(name: &'static str) -> TaskProfile {
        TaskProfile::new(name)
    }

    /// This function should be called a single time at the end of your game, once all other threads in your game finish.
    pub fn end_profiling() {
        let Profiler { sender, thread } = PROFILER
            .write()
            .take()
            .expect("Error: wtf::Profiler::end_profiling() already called");
        drop(sender);
        thread
            .join()
            .expect("Error: wtf::Profiler::end_profiling() failed to join thread");
    }

    fn send_message(msg: ProfilerMessage) {
        PROFILER
            .read()
            .as_ref()
            .expect("Error: wtf::Profiler::end_profiling() already called")
            .sender
            .send(msg)
            .unwrap();
    }
}

enum ProfilerMessage {
    Frame {
        elapsed: Duration,
    },
    TaskStart {
        name: &'static str,
    },
    TaskEnd {
        name: &'static str,
        elapsed: Duration,
    },
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

pub struct FrameProfile {
    start: Instant,
}

impl FrameProfile {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Drop for FrameProfile {
    fn drop(&mut self) {
        Profiler::send_message(ProfilerMessage::Frame {
            elapsed: self.start.elapsed(),
        });
    }
}

pub struct TaskProfile {
    name: &'static str,
    start: Instant,
}

impl TaskProfile {
    pub fn new(name: &'static str) -> Self {
        Profiler::send_message(ProfilerMessage::TaskStart { name });
        Self {
            name,
            start: Instant::now(),
        }
    }
}

impl Drop for TaskProfile {
    fn drop(&mut self) {
        Profiler::send_message(ProfilerMessage::TaskEnd {
            name: self.name,
            elapsed: self.start.elapsed(),
        });
    }
}

pub fn profile_data_from_bytes(bytes: &[u8]) -> Result<Box<[FrameData]>, Box<dyn Error>> {
    let total_bytes = bytes.len() as u64;
    let mut bytes = FrameDecoder::new(Cursor::new(bytes));
    let mut frames = Vec::new();
    while bytes.get_ref().position() < total_bytes - 1 {
        let frame: FrameData = bincode::deserialize_from(&mut bytes)?;
        frames.push(frame);
    }
    Ok(frames.into_boxed_slice())
}
