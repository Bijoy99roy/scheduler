use scheduler::engine::TimePriorityEngine;
use scheduler::job::Job;
use scheduler::queue::QueueManager;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Initializing Scheduler Component...");

    let queue = Arc::new(Mutex::new(QueueManager::new()));
    // Channel from the Time & Priority Engine to the Worker Executor
    let (tx, rx) = mpsc::channel();

    let engine = TimePriorityEngine::new(Arc::clone(&queue), tx);
    engine.start();

    // Start a simple worker simulation thread
    thread::spawn(move || {
        while let Ok(job) = rx.recv() {
            println!("[Worker] Executing job {} ('{}')", job.id, job.description);
            thread::sleep(Duration::from_millis(50)); // Simulating work
        }
    });

    // Schedule some jobs
    let now = chrono::Utc::now().timestamp();

    if let Ok(mut q) = queue.lock() {
        // Job due in 1 second, priority 5
        if let Ok(j1) = Job::new(now + 1, 5, "Backup Database", "backup_fn") {
            q.push(j1);
        }
        // Job due in 3 seconds, priority 1
        if let Ok(j2) = Job::new(now + 3, 1, "Send Emails", "email_fn") {
            q.push(j2);
        }
        // Job due in 1 second, priority 1 (Higher priority than j1)
        if let Ok(j3) = Job::new(now + 1, 1, "Urgent Hotfix", "hotfix_fn") {
            q.push(j3);
        }
    }

    println!("Jobs scheduled into Queue. Waiting for Engine to process...");

    // Wait enough time for all jobs to be processed
    thread::sleep(Duration::from_secs(4));
    println!("Scheduler simulation complete. Shutting down.");

    // Stop the engine gracefully
    engine.stop();
}
