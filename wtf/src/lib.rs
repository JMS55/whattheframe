use flume::Sender;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use snap::read::FrameDecoder;
use snap::write::FrameEncoder;
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Profiler {
    sender: Sender<ProfilerMessage>,
    thread: JoinHandle<()>,
}

static FRAME_NUMBER: AtomicU64 = AtomicU64::new(0);
static PROFILER: Lazy<RwLock<Option<Profiler>>> = Lazy::new(|| {
    let (sender, reciever) = flume::unbounded();

    let thread = thread::spawn(move || {
        let file_name = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file = File::create(format!("{}.wtf", file_name))
            .expect("Error: Failed to create wtf::Profiler file");
        let mut file = FrameEncoder::new(file);

        #[derive(Serialize)]
        struct TaskDataS<'a> {
            pub name: &'a str,
            pub duration: Duration,
            pub subtasks: Vec<Self>,
        }

        let mut frame = TaskDataS {
            name: "",
            duration: Duration::default(),
            subtasks: Vec::new(),
        };
        let mut parent_stack = Vec::new();
        parent_stack.push(&mut frame);

        for msg in reciever.iter() {
            match msg {
                ProfilerMessage::TaskStart { name } => {
                    let task = TaskDataS {
                        name,
                        duration: Duration::default(),
                        subtasks: Vec::new(),
                    };

                    let parent = *parent_stack.last().unwrap();
                    parent.subtasks.push(task);

                    let task_ref = parent.subtasks.last_mut().unwrap();
                    parent_stack.push(task_ref);
                }
                ProfilerMessage::TaskEnd { elapsed } => {
                    let task = parent_stack.last().unwrap();
                    task.duration = elapsed;
                    if parent_stack.len() == 1 {
                        let frame_number = FRAME_NUMBER.fetch_add(1, Ordering::SeqCst) + 1;
                        task.name = &format!("Frame: #{}", frame_number);

                        bincode::serialize_into(&mut file, task).unwrap();

                        frame.subtasks.clear();
                    } else {
                        parent_stack.pop().unwrap();
                    }
                }
            }
        }
    });

    RwLock::new(Some(Profiler { sender, thread }))
});

impl Profiler {
    /// This function should not be called after [`Profiler::end_profiling`].
    pub fn new_frame() -> TaskRecording {
        TaskRecording::new("")
    }

    /// This function should not be called after [`Profiler::end_profiling`].
    pub fn profile_task(name: &'static str) -> TaskRecording {
        TaskRecording::new(name)
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
        let profiler_lock = PROFILER.read();
        let r = profiler_lock
            .as_ref()
            .expect("Error: wtf::Profiler::end_profiling() was called")
            .sender
            .send(msg);
        drop(profiler_lock);
        r.unwrap();
    }
}

pub type ProfileData = Box<[TaskData]>;

pub fn profile_data_from_bytes(bytes: &[u8]) -> Result<ProfileData, Box<dyn Error>> {
    let total_bytes = bytes.len() as u64;
    let mut bytes = FrameDecoder::new(Cursor::new(bytes));
    let mut frames = Vec::new();
    while bytes.get_ref().position() < total_bytes - 1 {
        let frame: TaskData = bincode::deserialize_from(&mut bytes)?;
        frames.push(frame);
    }
    Ok(frames.into_boxed_slice())
}

#[derive(Clone, Deserialize)]
pub struct TaskData {
    pub name: Box<str>,
    pub duration: Duration,
    pub subtasks: Box<[Self]>,
}

enum ProfilerMessage {
    TaskStart { name: &'static str },
    TaskEnd { elapsed: Duration },
}

pub struct TaskRecording {
    start: Instant,
}

impl TaskRecording {
    fn new(name: &'static str) -> Self {
        Profiler::send_message(ProfilerMessage::TaskStart { name });
        Self {
            start: Instant::now(),
        }
    }
}

impl Drop for TaskRecording {
    fn drop(&mut self) {
        Profiler::send_message(ProfilerMessage::TaskEnd {
            elapsed: self.start.elapsed(),
        });
    }
}
