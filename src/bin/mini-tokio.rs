use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll},
    thread,
    time::{Duration, Instant},
};

use futures::task;

fn main() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };

        let out = future.await;
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            // Get a handleto the waker for the current task
            let waker = cx.waker().clone();
            let when = self.when;

            // Spawn a timer thread
            // wake() once enough time has elapsed
            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                waker.wake();
            });

            Poll::Pending
        }
    }
}

struct MiniTokio {
    tasks: VecDeque<Task>,
}

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

impl MiniTokio {
    fn new() -> Self {
        MiniTokio {
            tasks: VecDeque::new(),
        }
    }

    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push_back(Box::pin(future));
    }

    fn run(&mut self) {
        // What are these?
        let waker = task::noop_waker();
        let mut ctx = Context::from_waker(&waker);

        // See the task in the head of queue
        while let Some(mut task) = self.tasks.pop_front() {
            // Is the task is terminated?
            if task.as_mut().poll(&mut ctx).is_pending() {
                // No, see this task again later
                self.tasks.push_back(task);
            }
        }
    }
}
