use std::{
    pin::Pin,
    sync::{Arc, Mutex, mpsc},
    task::{Context, Poll, Waker},
    thread,
    time::{Duration, Instant},
};

use futures::task::{self, ArcWake};

fn main() {
    let mut mini_tokio = MiniTokio::new();
    let when = Instant::now() + Duration::from_millis(10);
    let future = Delay { when, waker: None };
    mini_tokio.spawn(future);

    mini_tokio.run();
}

struct Delay {
    when: Instant,
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.when {
            // Task finish!!
            println!("Hello world");
            return Poll::Ready(());
        }

        // The task has not finished.
        // Check if the thread is already spawned.
        // A waker is registered when the thread is spawned for the first time.
        if let Some(waker) = &self.waker {
            // This is not the first time
            let mut waker = waker.lock().unwrap();

            // Check if the current waker of context (cx.waker()) is
            // the registered waker (self.waker)
            if !waker.will_wake(cx.waker()) {
                // If not matches, update the registered waker
                *waker = cx.waker().clone();
            }
        } else {
            // This is the first time, so let's spawn a thread

            // Register a waker
            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());

            thread::spawn(move || {
                let now = Instant::now();
                if now < when {
                    thread::sleep(when - now);
                }

                // The task has finished.
                // Notify the caller by invoking the waker.
                let waker = waker.lock().unwrap();
                waker.wake_by_ref();
            });
        }

        Poll::Pending
    }
}

struct MiniTokio {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        MiniTokio {
            scheduled: rx,
            sender: tx,
        }
    }

    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&mut self) {
        // See the task in the head of queue
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}

struct Task {
    // Real tokio doesn't use Mutex, but the idea is the same
    task_future: Mutex<TaskFuture>,
    executor: mpsc::Sender<Arc<Task>>,
}

impl Task {
    fn poll(self: Arc<Self>) {
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut task_future = self.task_future.try_lock().unwrap();

        task_future.poll(&mut cx);
    }

    fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            task_future: Mutex::new(TaskFuture::new(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }

    fn schedule(self: &Arc<Self>) {
        self.executor.send(self.clone()).unwrap();
    }
}

// impl a handler for 'wake'
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // push the task into the channel
        arc_self.schedule();
    }
}
struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    poll: Poll<()>, // to keep track of the latest poll result
}

impl TaskFuture {
    fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        if self.poll.is_pending() {
            // Update the result of polling
            self.poll = self.future.as_mut().poll(cx);
        }
    }
}
