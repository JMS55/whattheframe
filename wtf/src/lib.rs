use flume::Sender;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use snap::read::FrameDecoder;
use snap::write::FrameEncoder;
use std::fs::File;
use std::io::{self, Read, Write};
use std::mem;
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
        let mut parent_stack: Vec<*mut TaskDataS> = vec![&mut frame];

        for msg in reciever.iter() {
            match msg {
                ProfilerMessage::TaskStart { name } => {
                    let task = TaskDataS {
                        name,
                        duration: Duration::default(),
                        subtasks: Vec::new(),
                    };

                    let parent = *parent_stack.last().unwrap();
                    let parent_subtasks = unsafe { &mut (*parent).subtasks };

                    parent_subtasks.push(task);
                    let task_ref = parent_subtasks.last_mut().unwrap();
                    parent_stack.push(task_ref);
                }
                ProfilerMessage::TaskEnd { elapsed } => {
                    let task = *parent_stack.last().unwrap();
                    unsafe { (*task).duration = elapsed }

                    if parent_stack.len() == 1 {
                        let frame_number = FRAME_NUMBER.fetch_add(1, Ordering::SeqCst) + 1;
                        let frame_name = format!("Frame: #{}", frame_number);

                        // SAFETY: frame.name is temporarily set to reference a local string.
                        // It must be reset to a valid reference by the end of the scope.
                        frame.name = unsafe { mem::transmute(frame_name.as_str()) };
                        bincode::serialize_into(&mut file, &frame).unwrap();
                        frame.name = "";

                        frame.subtasks.clear();
                    } else {
                        parent_stack.pop().unwrap();
                    }
                }
            }
        }

        file.flush().unwrap();
    });

    RwLock::new(Some(Profiler { sender, thread }))
});

impl Profiler {
    /// This function should not be called after [`Profiler::end_profiling`].
    pub fn new_frame() -> TaskRecording {
        TaskRecording {
            start: Instant::now(),
        }
    }

    /// This function should not be called after [`Profiler::end_profiling`].
    pub fn profile_task(name: &'static str) -> TaskRecording {
        Profiler::send_message(ProfilerMessage::TaskStart { name });
        TaskRecording {
            start: Instant::now(),
        }
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
        let sender = profiler_lock
            .as_ref()
            .expect("Error: wtf::Profiler::end_profiling() was called")
            .sender
            .clone();
        drop(profiler_lock);
        sender.send(msg).unwrap();
    }
}

pub type ProfileData = Box<[TaskData]>;

pub fn read_profile_data<R: Read>(reader: R) -> Result<ProfileData, bincode::Error> {
    let mut reader = FrameDecoder::new(reader);
    let mut frames = Vec::new();
    loop {
        let frame = bincode::deserialize_from(&mut reader);
        let frame = match frame {
            Ok(task) => task,
            Err(err) => match *err {
                bincode::ErrorKind::Io(err) if err.kind() == io::ErrorKind::UnexpectedEof => {
                    break;
                }
                _ => return Err(err),
            },
        };
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

impl Drop for TaskRecording {
    fn drop(&mut self) {
        Profiler::send_message(ProfilerMessage::TaskEnd {
            elapsed: self.start.elapsed(),
        });
    }
}
