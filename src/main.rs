use scheduler::engine::TimePriorityEngine;
use scheduler::job::Job;
use scheduler::queue::QueueManager;
use scheduler::telemetry;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

fn main() {
    let _guard = telemetry::init_telemetry();
    tracing::info!("Scheduler Component Initialized!");
    telemetry::log_resource_usage();

    let queue = Arc::new(Mutex::new(QueueManager::new()));
    let (tx, rx) = mpsc::channel();

    let engine = TimePriorityEngine::new(Arc::clone(&queue), tx);
    engine.start();

    thread::spawn(move || {
        while let Ok(mut job) = rx.recv() {
            job.start();
            tracing::info!("[Worker] Executing job {} ('{}')", job.id, job.description);
            thread::sleep(Duration::from_millis(50));
            job.complete();
        }
    });

    let now = chrono::Utc::now().timestamp();

    if let Ok(mut q) = queue.lock() {
        if let Ok(j1) = Job::new(now + 1, 5, "Backup Database", "backup_fn", 3) {
            q.push(j1);
        }
        if let Ok(j2) = Job::new(now + 3, 1, "Send Emails", "email_fn", 1) {
            q.push(j2);
        }
        if let Ok(j3) = Job::new(now + 1, 1, "Urgent Hotfix", "hotfix_fn", 3) {
            q.push(j3);
        }
    }

    tracing::info!("Jobs scheduled. Waiting for Engine to process...");
    thread::sleep(Duration::from_secs(4));

    telemetry::log_resource_usage();
    tracing::info!("Simulation complete. Shutting down.");

    engine.stop();
}
