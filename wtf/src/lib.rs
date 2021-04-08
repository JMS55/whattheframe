use serde::Deserialize;
use snap::read::FrameDecoder;
use std::io::{self, Read};
use std::time::Duration;

#[cfg(feature = "profile")]
use {
    chrono::offset::Utc,
    flume::Sender,
    once_cell::sync::Lazy,
    serde::Serialize,
    snap::write::FrameEncoder,
    std::env,
    std::fs::File,
    std::io::Write,
    std::mem,
    std::sync::RwLock,
    std::thread::{self, JoinHandle},
    std::time::Instant,
};

#[cfg(feature = "profile")]
static PROFILER: Lazy<Profiler> = Lazy::new(|| {
    let (sender, reciever) = flume::unbounded();

    let thread = thread::Builder::new()
        .name("wtf-profiler".to_string())
        .spawn(move || {
            let program_name = env::current_exe().unwrap();
            let program_name = program_name.file_name().unwrap().to_str().unwrap();
            let timestamp = Utc::now().format("%F-%T");
            let file = File::create(format!("{}-{}.wtf", program_name, timestamp)).unwrap();
            let mut file = FrameEncoder::new(file);

            #[derive(Serialize)]
            struct TaskDataS<'a> {
                name: &'a str,
                duration: Duration,
                subtasks: Vec<Self>,
            }

            let mut frame_number = 0;
            let mut frame = TaskDataS {
                name: "",
                duration: Duration::default(),
                subtasks: Vec::new(),
            };
            let mut parent_stack: Vec<*mut TaskDataS> = vec![&mut frame];

            loop {
                let msg = reciever.recv_timeout(Duration::from_millis(100));
                match msg {
                    Ok(ProfilerMessage::TaskStart { name }) => {
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
                    Ok(ProfilerMessage::TaskEnd { elapsed }) => {
                        let task = *parent_stack.last().unwrap();
                        unsafe { (*task).duration = elapsed }

                        if parent_stack.len() == 1 {
                            frame_number += 1;
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
                    _ => {
                        if PROFILER.thread.read().unwrap().is_none() {
                            break;
                        }
                    }
                }
            }

            file.flush().unwrap();
        })
        .unwrap();
    let thread = RwLock::new(Some(thread));

    Profiler { sender, thread }
});

pub struct Profiler {
    #[cfg(feature = "profile")]
    sender: Sender<ProfilerMessage>,
    #[cfg(feature = "profile")]
    thread: RwLock<Option<JoinHandle<()>>>,
}

#[cfg(feature = "profile")]
pub type ProfilingReturnType = TaskRecord;
#[cfg(not(feature = "profile"))]
pub type ProfilingReturnType = ();

impl Profiler {
    /// This function should not be called after [`Profiler::end_profiling`].
    #[must_use = "Must assign to a variable: \"_record = new_frame()\""]
    pub fn new_frame() -> ProfilingReturnType {
        #[cfg(feature = "profile")]
        TaskRecord {
            start: Instant::now(),
        }
    }

    /// This function should not be called after [`Profiler::end_profiling`].
    #[must_use = "Must assign to a variable: \"_record = profile_task()\""]
    #[allow(unused_variables)]
    pub fn profile_task(name: &'static str) -> ProfilingReturnType {
        #[cfg(feature = "profile")]
        {
            PROFILER
                .sender
                .send(ProfilerMessage::TaskStart { name })
                .unwrap();
            TaskRecord {
                start: Instant::now(),
            }
        }
    }

    // This function should be called a single time at the end of your game, once all other threads in your game finish.
    pub fn end_profiling() {
        #[cfg(feature = "profile")]
        {
            let mut profiler_lock = PROFILER.thread.write().unwrap();
            let thread = profiler_lock.take().unwrap();
            drop(profiler_lock);
            thread.join().unwrap();
        }
    }
}

pub type ProfileData = Box<[TaskData]>;

pub fn read_profile_data<R: Read>(reader: R) -> Result<ProfileData, bincode::Error> {
    let mut reader = FrameDecoder::new(reader);
    let mut frames = Vec::new();
    loop {
        let frame = bincode::deserialize_from(&mut reader);
        let frame = match frame.map_err(|err| *err) {
            Ok(task) => task,
            Err(bincode::ErrorKind::Io(err)) if err.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(Box::new(err)),
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

#[cfg(feature = "profile")]
enum ProfilerMessage {
    TaskStart { name: &'static str },
    TaskEnd { elapsed: Duration },
}

#[cfg(feature = "profile")]
pub struct TaskRecord {
    start: Instant,
}

#[cfg(feature = "profile")]
impl Drop for TaskRecord {
    fn drop(&mut self) {
        PROFILER
            .sender
            .send(ProfilerMessage::TaskEnd {
                elapsed: self.start.elapsed(),
            })
            .unwrap();
    }
}
