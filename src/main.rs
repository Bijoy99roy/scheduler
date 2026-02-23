use scheduler::engine::TimePriorityEngine;
use scheduler::job::Job;
use scheduler::persistence_manager::PersistenceManager;
use scheduler::queue::QueueManager;
use scheduler::telemetry;
use scheduler::tui;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

fn main() -> std::io::Result<()> {
    // Load .env for RESEND_API_KEY, SMTP_FROM, SMTP_RECIPIENT
    dotenvy::dotenv().ok();

    // Initialize Telemetry
    let _guard = telemetry::init_telemetry();
    tracing::info!("Scheduler Component Initialized!");
    telemetry::log_resource_usage();

    let persistence = PersistenceManager::new("queue.json");
    let loaded_jobs = persistence.load_jobs();

    let mut q = QueueManager::new();
    q.load_from_vec(loaded_jobs);
    let snapshot_tx = persistence.start_memory_snapshot();
    q.set_persistence(snapshot_tx);

    let queue = Arc::new(Mutex::new(q));

    // Channel from the Time & Priority Engine to the Worker Executor
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

    // Previous code: schedule demo jobs only if queue is empty
    if queue.lock().unwrap().is_empty() {
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
    }

    tracing::info!("Jobs scheduled. Starting TUI...");
    telemetry::log_resource_usage();

    let result = tui::run_tui(
        queue,
        log_rx,
        worker_tx,
        vec!["backup_fn".into(), "email_fn".into(), "hotfix_fn".into()],
    );
    engine.stop();
    result
}
