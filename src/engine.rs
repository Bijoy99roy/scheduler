use crate::job::{Job, Status};
use crate::queue::QueueManager;
use chrono::Utc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct TimePriorityEngine {
    queue: Arc<Mutex<QueueManager>>,
    worker_tx: Sender<Job>,
    is_running: Arc<AtomicBool>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl TimePriorityEngine {
    pub fn new(queue: Arc<Mutex<QueueManager>>, worker_tx: Sender<Job>) -> Self {
        Self {
            queue,
            worker_tx,
            is_running: Arc::new(AtomicBool::new(false)),
            handle: Mutex::new(None),
        }
    }

    /// Starts the Time & Priority Engine in a background thread.
    /// It polls the queue at a set interval for jobs that are ready to execute.
    pub fn start(&self) {
        let mut handle_lock = self.handle.lock().unwrap();
        if handle_lock.is_some() {
            println!("[Engine] Already running.");
            return;
        }

        self.is_running.store(true, Ordering::SeqCst);
        let queue_clone = Arc::clone(&self.queue);
        let tx_clone = self.worker_tx.clone();
        let running_flag = Arc::clone(&self.is_running);

        let thread_handle = thread::spawn(move || {
            println!("[Engine] Started polling thread.");
            while running_flag.load(Ordering::Relaxed) {
                let now = Utc::now().timestamp();

                let mut ready_jobs = Vec::new();
                // Secure the lock briefly to extract ready jobs
                if let Ok(mut q) = queue_clone.lock() {
                    ready_jobs = q.pop_ready(now);
                }

                // Push ready jobs to the worker channel
                for mut job in ready_jobs {
                    job.status = Status::Running;
                    println!(
                        "[Engine] Job {} ('{}') is ready (priority: {}). Dispatching to worker...",
                        job.id, job.description, job.priority
                    );
                    if let Err(e) = tx_clone.send(job) {
                        eprintln!("[Engine] Failed to dispatch job: {}", e);
                    }
                }

                // Poll every 500ms
                thread::sleep(Duration::from_millis(500));
            }
            println!("[Engine] Polling thread stopped gracefully.");
        });

        *handle_lock = Some(thread_handle);
    }

    /// Signals the Engine thread to stop and waits for it to finish gracefully.
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        let mut handle_lock = self.handle.lock().unwrap();
        if let Some(handle) = handle_lock.take() {
            let _ = handle.join();
        }
    }
}
