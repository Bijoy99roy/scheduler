use std::collections::HashMap;

use crate::job::Job;

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
    pub fn run_job(&self, job: &mut Job) {
        if let Some(func) = self.registry.get(&job.function) {
            job.start();
            println!("[Worker] Executing: {}", job.function);
            func(); // Execute the function pointer
            job.complete();
        } else {
            eprintln!(
                "[Worker] Error: No function registered for '{}'",
                job.function
            );
            job.fail_and_retry();
        }
    }

    /// Starts a simple blocking loop to process jobs from the channel
    pub fn start(&self, rx: std::sync::mpsc::Receiver<Job>) {
        for mut job in rx {
            self.run_job(&mut job);
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
