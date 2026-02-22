use scheduler::{
    job::{Job, Status},
    worker::Worker,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;
    // We use a static atomic to track if the function was called
    static WAS_CALLED: AtomicBool = AtomicBool::new(false);

    fn test_task(_log_tx: std::sync::mpsc::Sender<String>) {
        WAS_CALLED.store(true, Ordering::SeqCst);
    }

    #[test]
    fn test_worker_registry_execution() {
        let mut worker = Worker::new();

        // 1. Register our test function
        worker.register("test_func", test_task);

        // 2. Create a job that is ready to run (execution_time = 0)
        let job = Job {
            id: uuid::Uuid::new_v4(),
            function: "test_func".to_string(),
            description: "A test job for the registry".to_string(),
            priority: 1,
            execution_time: 0,
            status: Status::Pending,
        };

        // 3. Reset the flag and run the job
        let (log_tx, _log_rx) = mpsc::channel();
        WAS_CALLED.store(false, Ordering::SeqCst);
        worker.run_job(&job, log_tx);

        // 4. Assert the function was triggered
        assert!(
            WAS_CALLED.load(Ordering::SeqCst),
            "The registered function should have been executed"
        );
    }

    #[test]
    fn test_unknown_function_graceful_failure() {
        let worker = Worker::new(); // No functions registered

        let job = Job {
            id: uuid::Uuid::new_v4(),
            function: "missing_func".to_string(),
            description: "A test job for the registry".to_string(),
            priority: 2,
            execution_time: 0,
            status: Status::Pending,
        };

        // Should not panic, just log an error
        let (log_tx, _log_rx) = mpsc::channel();
        worker.run_job(&job, log_tx);
    }

    #[test]
    fn test_worker_start_channel() {
        let mut worker = Worker::new();
        worker.register("test_func", test_task);

        let (tx, rx) = mpsc::channel();
        WAS_CALLED.store(false, Ordering::SeqCst);

        let (log_tx, _log_rx) = mpsc::channel();

        // Start worker in a thread
        thread::spawn(move || {
            worker.start(rx, log_tx);
        });

        let job = Job {
            id: uuid::Uuid::new_v4(),
            function: "test_func".to_string(),
            description: "Test channel job".to_string(),
            priority: 1,
            execution_time: 0,
            status: Status::Pending,
        };

        tx.send(job).unwrap();

        // Wait a bit for the thread to process the message
        thread::sleep(Duration::from_millis(50));

        // Assert the function was executed
        assert!(
            WAS_CALLED.load(Ordering::SeqCst),
            "The registered function should have been executed via channel"
        );
    }
}
