use chrono::Utc;
use scheduler::{engine::TimePriorityEngine, job::Job, queue::QueueManager};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

#[test]
fn engine_dispatches_jobs_on_time() {
    let queue = Arc::new(Mutex::new(QueueManager::new()));
    let (tx, rx) = mpsc::channel();
    let engine = TimePriorityEngine::new(Arc::clone(&queue), tx);

    let now = Utc::now().timestamp();

    {
        let mut q = queue.lock().unwrap();
        // A job exactly at 'now'
        q.push(Job::new(now, 1, "now job", "fn").unwrap());
        // A job 1 second in the future
        q.push(Job::new(now + 1, 1, "future job", "fn").unwrap());
    }

    engine.start();

    // The 'now' job should arrive almost immediately (within 1 second)
    let job1 = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(job1.description, "now job");

    // The 'future' job should arrive after its time point (we give it up to 2 seconds to account for sleeping)
    let job2 = rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(job2.description, "future job");

    engine.stop();
}
