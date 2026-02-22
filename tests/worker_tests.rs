use scheduler::{job::{Job, Status}, queue::QueueManager, worker::Worker};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

#[cfg(test)]
mod tests {
    use super::*;
    // We use a static atomic to track if the function was called
    static WAS_CALLED: AtomicBool = AtomicBool::new(false);

    fn test_task() {
        WAS_CALLED.store(true, Ordering::SeqCst);
    }

    #[test]
    fn test_worker_registry_execution() {
        let manager = Arc::new(Mutex::new(QueueManager::new()));
        let mut worker = Worker::new();
        // 1. Register our test function
        worker.register("test_func", test_task);

        // 2. Create a job that is ready to run (execution_time = 0)
        let job = Job {
            id: uuid::Uuid::new_v4(),
            function: "test_func".to_string(),
            description: "A test job for the registry".to_string(), // Added
            priority: 1,
            execution_time: 0, 
            status: Status::Pending,
        };

        {
            let mut lock = manager.lock().unwrap();
            lock.push(job);
        }

        WAS_CALLED.store(false, Ordering::SeqCst);
        worker.process_once(Arc::clone(&manager));

        assert!(WAS_CALLED.load(Ordering::SeqCst), "The registered function should have been executed");
        
        let lock = manager.lock().unwrap();
        assert_eq!(lock.len(), 0, "The job should have been popped from the queue");
    }

    #[test]
    fn test_unknown_function_graceful_failure() {
        let manager = Arc::new(Mutex::new(QueueManager::new()));
        let worker = Worker::new(); 

        let job = Job {
            id: uuid::Uuid::new_v4(),
            function: "missing_func".to_string(),
            description: "Unknown task".to_string(),
            priority: 2,
            execution_time: 0,
            status: Status::Pending,
        };
        
        manager.lock().unwrap().push(job);

        // Should not panic, just log an error
        worker.process_once(Arc::clone(&manager));
        
        assert_eq!(manager.lock().unwrap().len(), 0, "The job should still be popped");
    }
}