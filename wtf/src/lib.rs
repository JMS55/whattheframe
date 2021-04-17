//! A library for profiling frame-based games.
//!
//! After recording a profile, you can view it in the [WhatTheFrame GUI](https://github.com/JMS55/whattheframe).
//!
//!
//! # Activating Profiling
//!
//! By default, [`Profiler::new_frame`], [`Profiler::profile_task`], and [`Profiler::end_profiling`] will compile to no-ops.
//!
//! To enable profiling, turn on the `profile` feature. You probably want to configure your game's `Cargo.toml` like so:
//! ```toml
//! [features]
//! profile = ["wtf/profile"]
//! [dependencies]
//! wtf = "1.0"
//! ```
//!
//! And then run your game like so:
//!
//! `cargo run --features profile`
//!
//!
//! # API
//!
//! The API consists of 4 functions:
//! * [`read_profile_data`] - Used to read a `.wtf` profile.
//! * [`Profiler::new_frame`] - Call at the start of your frame
//! * [`Profiler::profile_task`] - Call at the top of each scope you want to profile
//! * [`Profiler::end_profiling`] - Call _once_ at the end of your game
//!
//! Note that you _must_ assign [`Profiler::new_frame`] and [`Profiler::profile_task`] to a variable (_not_ `_`) like so:
//! ```rust
//! let _profile = Profiler::new_frame();
//! ```
//! ```rust
//! let _profile = Profiler::new_task("foo");
//! ```
//!
//! Example with `winit`:
//! ```rust
//! use std::sync::atomic::{AtomicBool, Ordering};
//! use std::sync::Arc;
//! use std::thread;
//! use winit::event::Event;
//! use wtf::Profiler;
//!
//! fn main() {
//!     let mut profiler_frame = None;
//!
//!     let thread_should_quit = Arc::new(AtomicBool::new(false));
//!     let thread = thread::spawn({
//!         let thread_should_quit = Arc::clone(&thread_should_quit);
//!
//!         move || {
//!             while !thread_should_quit.load(Ordering::SeqCst) {
//!                 {
//!                     let _profile = Profiler::profile_task("thread_part_1");
//!                     thread_task_1();
//!                     thread_task_2();
//!                     thread_task_3();
//!                 }
//!
//!                 {
//!                     let _profile = Profiler::profile_task("thread_part_2");
//!                     thread_task_4();
//!                     thread_task_5();
//!                 }
//!             }
//!         }
//!     });
//!
//!     event_loop.run(move |event, _, _| match &event {
//!         Event::NewEvents(_) => {
//!             profiler_frame = Some(Profiler::new_frame());
//!         }
//!
//!         Event::MainEventsCleared => {
//!             let record = Profiler::profile_task("update_game");
//!             update_state();
//!             update_positions();
//!             drop(record);
//!
//!             window.request_redraw();
//!         }
//!
//!         Event::RedrawEventsCleared => {
//!             let frame = profiler_frame.take();
//!             drop(frame);
//!         }
//!
//!         Event::LoopDestroyed => {
//!             thread_should_quit.store(true, Ordering::SeqCst);
//!             thread.join().unwrap();
//!
//!             Profiler::end_profiling();
//!         }
//!
//!         _ => {}
//!     });
//! }
//! ```

use serde::Deserialize;
use snap::read::FrameDecoder;
use std::io::{self, Read};
use std::time::Duration;

#[cfg(feature = "profile")]
use {
    bumpalo::Bump as Arena,
    chrono::offset::Utc,
    flume::Sender,
    once_cell::sync::Lazy,
    serde::Serialize,
    snap::write::FrameEncoder,
    std::env,
    std::fs::File,
    std::io::Write,
    std::mem,
    std::sync::Mutex,
    std::thread::{self, JoinHandle},
    std::time::Instant,
};

#[cfg(feature = "profile")]
static PROFILER: Lazy<Profiler> = Lazy::new(|| {
    let (sender, reciever) = flume::unbounded();

    let thread = thread::Builder::new()
        .name("wtf-profiler".to_string())
        .spawn(move || {
            fn create_file() -> Option<FrameEncoder<File>> {
                let program_name = env::current_exe().ok()?;
                let program_name = program_name.file_name()?.to_str()?;
                let timestamp = Utc::now().format("%F-%T");
                let file = File::create(format!("{}-{}.wtf", program_name, timestamp)).ok()?;
                Some(FrameEncoder::new(file))
            }
            let mut file = create_file().expect("WTF: Failed to create file for profile");

            #[derive(Serialize)]
            struct TaskDataS<'a> {
                name: &'a str,
                duration: Duration,
                subtasks: Vec<&'a Self>,
            }

            let mut task_arena = Arena::with_capacity(1000);
            let mut frame_number: usize = 0;
            let mut frame = TaskDataS {
                name: "",
                duration: Duration::default(),
                subtasks: Vec::with_capacity(10),
            };
            let mut parent_stack: Vec<*mut TaskDataS> = vec![&mut frame];

            loop {
                let msg = reciever.recv_timeout(Duration::from_millis(100));
                match msg {
                    Ok(ProfilerMessage::TaskStart { name }) => {
                        // Create a new task with a placeholder duration
                        let task = TaskDataS {
                            name,
                            duration: Duration::default(),
                            subtasks: Vec::new(),
                        };
                        let task_mut_ref: *mut TaskDataS = task_arena.alloc(task);
                        let task_ref = unsafe { &*task_mut_ref };

                        let parent = *parent_stack.last().unwrap();
                        let parent_subtasks = unsafe { &mut (*parent).subtasks };

                        // Add it to the current parent's subtasks
                        parent_subtasks.push(task_ref);

                        // Push it to the top of the parent stack
                        parent_stack.push(task_mut_ref);
                    }
                    Ok(ProfilerMessage::TaskEnd { elapsed }) => {
                        // Replace the placeholder with the real duration
                        let task = *parent_stack.last().unwrap();
                        unsafe { (*task).duration = elapsed }

                        // If only 1 task left in the parent stack (the frame), then write the frame to the file
                        // Else pop the parent stack
                        if parent_stack.len() == 1 {
                            frame_number += 1;
                            let frame_name = format!("Frame #{}", frame_number);
                            // SAFETY: frame.name is temporarily set to reference a local string
                            // It must be reset to a valid reference by the end of the scope
                            frame.name = unsafe { mem::transmute(frame_name.as_str()) };

                            bincode::serialize_into(&mut file, &frame)
                                .expect("WTF: Failed to write data to file");

                            // Reset for the next frame
                            frame.name = "";
                            frame.subtasks.clear();
                            task_arena.reset();
                        } else {
                            parent_stack.pop().unwrap();
                        }
                    }
                    _ => {
                        // Haven't recieved any data recently, check if thread should finish
                        if PROFILER
                            .thread
                            .lock()
                            .expect("WTF: Failed to acquire thread lock")
                            .is_none()
                        {
                            break;
                        }
                    }
                }
            }

            file.flush().expect("WTF: Failed to write data to file");
        })
        .expect("WTF: Failed to spawn data writing thread");
    let thread = Mutex::new(Some(thread));

    Profiler { sender, thread }
});

pub struct Profiler {
    #[cfg(feature = "profile")]
    sender: Sender<ProfilerMessage>,
    #[cfg(feature = "profile")]
    thread: Mutex<Option<JoinHandle<()>>>,
}

#[cfg(feature = "profile")]
#[doc(hidden)]
pub type ProfilingReturnType = TaskRecord;
#[cfg(not(feature = "profile"))]
#[doc(hidden)]
pub type ProfilingReturnType = ();

impl Profiler {
    #[must_use = "Must assign to a variable: \"_profile = Profiler::new_frame()\""]
    pub fn new_frame() -> ProfilingReturnType {
        #[cfg(feature = "profile")]
        TaskRecord {
            start: Instant::now(),
        }
    }

    #[must_use = "Must assign to a variable: \"_profile = Profiler::profile_task()\""]
    #[allow(unused_variables)]
    pub fn profile_task(name: &'static str) -> ProfilingReturnType {
        #[cfg(feature = "profile")]
        {
            PROFILER
                .sender
                .send(ProfilerMessage::TaskStart { name })
                .expect("WTF: Failed to send task across a thread");
            TaskRecord {
                start: Instant::now(),
            }
        }
    }

    pub fn end_profiling() {
        #[cfg(feature = "profile")]
        {
            // Take the thread handle out, indicating to the data writing thread to stop
            let mut profiler_lock = PROFILER
                .thread
                .lock()
                .expect("WTF: Failed to acquire thread lock");
            let thread = profiler_lock
                .take()
                .expect("WTF: Profiler::end_profiling() has already been called once");
            // Drop the lock so the data writing thread can acquire it
            drop(profiler_lock);
            // Wait for the data writing thread to finish
            thread
                .join()
                .expect("WTF: Failed to join data writing thread");
        }
    }
}

pub type ProfileData = Box<[TaskData]>;

pub fn read_profile_data<R: Read>(reader: R) -> Result<ProfileData, bincode::Error> {
    let mut reader = FrameDecoder::new(reader);
    let mut frames = Vec::new();
    // Keep trying to read TaskData's until there aren't any more to read
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
        let msg = ProfilerMessage::TaskEnd {
            elapsed: self.start.elapsed(),
        };
        PROFILER
            .sender
            .send(msg)
            .expect("WTF: Failed to send task across a thread");
    }
}
