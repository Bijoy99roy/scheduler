use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::{thread, time::Duration};
use std::sync::{Arc, Mutex};

use crate::job::Job;
use crate::queue::QueueManager;

/// Type alias for a function pointer that takes no arguments and returns nothing
type JobFn = fn();

pub struct Worker {
    registry: HashMap<String, JobFn>,
}

impl Worker {
    /// Initialize a new worker with an empty registry
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Register a function string to a concrete function pointer
    pub fn register(&mut self, name: &str, f: JobFn) {
        self.registry.insert(name.to_string(), f);
    }

    /// The execution engine: looks up the string in the map and calls the function
    pub fn run_job(&self, job: &Job) {
        if let Some(func) = self.registry.get(&job.function) {
            println!("[Worker] Executing: {}", job.function);
            func(); // Execute the function pointer
        } else {
            eprintln!("[Worker] Error: No function registered for '{}'", job.function);
        }
    }

    /// Starts a simple polling loop to process jobs from the queue
    pub fn start(&self, queue: Arc<Mutex<QueueManager>>) {
        loop {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // Scope the lock so it is released as soon as we have the jobs
            let ready_jobs = {
                let mut lock = queue.lock().expect("Mutex poisoned");
                lock.pop_ready(now)
            }; // Lock drops here

            if ready_jobs.is_empty() {
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            for job in ready_jobs {
                self.run_job(&job);
            }
        }
    }

    /// Processes all currently ready jobs once and returns (for testing/manual polling)
    pub fn process_once(&self, queue: Arc<Mutex<QueueManager>>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        // 1. Acquire the lock and extract ready jobs
        let ready_jobs = {
            let mut lock = queue.lock().expect("Failed to lock QueueManager: Mutex poisoned");
            lock.pop_ready(now)
        }; // 2. Mutex lock is automatically dropped here

        // 3. Execute jobs outside of the lock to allow concurrency
        for job in ready_jobs {
            self.run_job(&job);
        }
    }
}

// --- Task Functions ---

pub fn send_email() {
    println!("📧 [Task] Sending email...");
    // Logic for sending email here
}

pub fn backup_db() {
    println!("🗄️ [Task] Backing up database...");
    // Logic for DB backup here
}