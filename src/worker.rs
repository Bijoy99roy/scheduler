use std::collections::HashMap;
use std::sync::mpsc::Sender;

use crate::job::Job;

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
            // Inject job metadata as env vars so task functions can read them
            if let Some(ref meta) = job.metadata {
                for (key, val) in meta {
                    // SAFETY: single-threaded worker processes jobs sequentially
                    unsafe { std::env::set_var(key, val); }
                }
            }
            job.start();
            let _ = log_tx.send(format!("[Worker] Executing '{}'", job.description));
            func(log_tx.clone()); // Execute the function
            job.complete();
            let _ = log_tx.send(format!("[Worker] Done '{}'", job.description));
        } else {
            let _ = log_tx.send(format!(
                "[Worker] Error: No function registered for '{}'",
                job.function
            ));
            job.fail_and_retry();
        }
    }

    /// Starts a simple blocking loop to process jobs from the channel
    pub fn start(&self, rx: std::sync::mpsc::Receiver<Job>, log_tx: Sender<String>) {
        for mut job in rx {
            self.run_job(&mut job, log_tx.clone());
        }
    }
}
