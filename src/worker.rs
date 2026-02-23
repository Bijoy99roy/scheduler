use std::collections::HashMap;
use std::sync::mpsc::Sender;

use crate::job::Job;
use notify_rust::Notification;

/// Type alias for a function pointer that takes log_tx and returns nothing
// type JobFn = fn(Sender<String>);

pub struct Worker {
    registry: HashMap<String, Box<dyn Fn(Sender<String>) + Send>>,
}

impl Worker {
    /// Initialize a new worker with an empty registry
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Register a function string to a concrete function
    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Sender<String>) + Send + 'static,
    {
        self.registry.insert(name.to_string(), Box::new(f));
    }

    /// The execution engine: looks up the string in the map and calls the function
    pub fn run_job(&self, job: &mut Job, log_tx: Sender<String>) {
        if let Some(func) = self.registry.get(&job.function) {
            job.start();
            let _ = log_tx.send(format!("[Worker] Executing '{}'", job.description));
            func(log_tx.clone()); // Execute the function
            job.complete();
            let _ = log_tx.send(format!("[Worker] Done '{}'", job.description));

            // Notify user of successful completion
            let _ = Notification::new()
                .summary("Task Scheduler")
                .body(&format!("Job '{}' completed successfully.", job.description))
                .show();
        } else {
            let _ = log_tx.send(format!(
                "[Worker] Error: No function registered for '{}'",
                job.function
            ));
            let will_retry = job.fail_and_retry();

            let msg = if will_retry {
                format!(
                    "Job '{}' failed. Will retry ({}/{})",
                    job.description, job.retry_count, job.max_retries
                )
            } else {
                format!("Job '{}' failed permanently.", job.description)
            };

            // Notify user of failure
            let _ = Notification::new()
                .summary("Task Scheduler")
                .body(&msg)
                .show();
        }
    }

    /// Starts a simple blocking loop to process jobs from the channel
    pub fn start(&self, rx: std::sync::mpsc::Receiver<Job>, log_tx: Sender<String>) {
        for mut job in rx {
            self.run_job(&mut job, log_tx.clone());
        }
    }
}

// --- Task Functions ---

pub fn send_email(log_tx: Sender<String>) {
    let _ = log_tx.send("📧 [Task] Sending email...".to_string());
    // Logic for sending email here
}

pub fn backup_db(log_tx: Sender<String>) {
    let _ = log_tx.send("🗄️ [Task] Backing up database...".to_string());
    // Logic for DB backup here
}
