use scheduler::engine::TimePriorityEngine;
use scheduler::job::Job;
use scheduler::queue::QueueManager;
use scheduler::tui;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

fn main() -> std::io::Result<()> {
    println!("Initializing Scheduler Component...");
    // Channel from the Time & Priority Engine to the Worker Executor
    let queue = Arc::new(Mutex::new(QueueManager::new()));
    let (worker_tx, worker_rx) = mpsc::channel();
    let (log_tx, log_rx) = mpsc::channel();

    let engine =
        TimePriorityEngine::new_with_log(Arc::clone(&queue), worker_tx.clone(), log_tx.clone());
    engine.start();

    // Start the real Worker in a separate thread
    thread::spawn(move || {
        let mut worker = scheduler::worker::Worker::new();
        // Register actual functions from worker.rs (or inline closures)
        worker.register("backup_fn", scheduler::worker::backup_db);
        worker.register("email_fn", scheduler::worker::send_email);
        worker.register("hotfix_fn", |log_tx: std::sync::mpsc::Sender<String>| {
            let _ = log_tx.send(" [Task] Applying urgent hotfix...".to_string());
        });

        worker.start(worker_rx, log_tx);
    });

    // Previous code: schedule demo jobs (unchanged logic)
    let now = chrono::Utc::now().timestamp();
    if let Ok(mut q) = queue.lock() {
        if let Ok(j1) = Job::new(now + 1, 5, "Backup Database", "backup_fn") {
            q.push(j1);
        }
        if let Ok(j2) = Job::new(now + 3, 1, "Send Emails", "email_fn") {
            q.push(j2);
        }
        if let Ok(j3) = Job::new(now + 1, 1, "Urgent Hotfix", "hotfix_fn") {
            q.push(j3);
        }
    }

    let result = tui::run_tui(
        queue,
        log_rx,
        worker_tx,
        vec!["backup_fn".into(), "email_fn".into(), "hotfix_fn".into()],
    );
    engine.stop();
    result
}
