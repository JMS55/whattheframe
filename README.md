A library for profiling frame-based games.

After recording a profile, you can view it in the [WhatTheFrame GUI](https://github.com/JMS55/whattheframe).


# Activating Profiling

By default, [`Profiler::new_frame`], [`Profiler::profile_task`], and [`Profiler::end_profiling`] will compile to no-ops.

To enable profiling, turn on the `profile` feature. You probably want to configure your game's `Cargo.toml` like so:
```toml
[features]
profile = ["wtf/profile"]
[dependencies]
wtf = "1.0"
```

And then run your game like so:

`cargo run --features profile`


# API

The API consists of 4 functions:
* [`read_profile_data`] - Used to read a `.wtf` profile
* [`Profiler::new_frame`] - Call at the start of your frame
* [`Profiler::profile_task`] - Call at the top of each scope you want to profile
* [`Profiler::end_profiling`] - Call _once_ at the end of your game

Note that you _must_ assign [`Profiler::new_frame`] and [`Profiler::profile_task`] to a variable (_not_ `_`) like so:
```rust
let _profile = Profiler::new_frame();
```
```rust
let _profile = Profiler::new_task("foo");
```

Example with `winit`:
```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use winit::event::Event;
use wtf::Profiler;

fn main() {
    let mut profiler_frame = None;

    let thread_should_quit = Arc::new(AtomicBool::new(false));
    let thread = thread::spawn({
        let thread_should_quit = Arc::clone(&thread_should_quit);

        move || {
            while !thread_should_quit.load(Ordering::SeqCst) {
                {
                    let _profile = Profiler::profile_task("thread_part_1");
                    thread_task_1();
                    thread_task_2();
                    thread_task_3();
                }

                {
                    let _profile = Profiler::profile_task("thread_part_2");
                    thread_task_4();
                    thread_task_5();
                }
            }
        }
    });

    event_loop.run(move |event, _, _| match &event {
        Event::NewEvents(_) => {
            profiler_frame = Some(Profiler::new_frame());
        }

        Event::MainEventsCleared => {
            let record = Profiler::profile_task("update_game");
            update_state();
            update_positions();
            drop(record);

            window.request_redraw();
        }

        Event::RedrawEventsCleared => {
            let frame = profiler_frame.take();
            drop(frame);
        }

        Event::LoopDestroyed => {
            thread_should_quit.store(true, Ordering::SeqCst);
            thread.join().unwrap();

            Profiler::end_profiling();
        }

        _ => {}
    });
}
```
