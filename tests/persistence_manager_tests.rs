#[cfg(test)]
mod tests {
    use scheduler::job::Job;
    use scheduler::job::Status;
    use scheduler::persistence_manager::PersistenceManager;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use uuid::Uuid;

    fn get_temp_path() -> String {
        format!("scheduler_jobs_{}.json", Uuid::new_v4())
    }

    #[test]
    fn test_load_non_existent_file_returns_empty() {
        let temp_file = get_temp_path();
        let manager = PersistenceManager::new(&temp_file);

        let jobs = manager.load_jobs();
        assert!(jobs.is_empty());
    }

    #[test]
    fn test_load_corrupted_file_returns_empty() {
        let temp_file = get_temp_path();

        fs::write(&temp_file, "{ randome stuff ]").unwrap();

        let manager = PersistenceManager::new(&temp_file);

        let jobs = manager.load_jobs();
        assert!(jobs.is_empty());

        fs::remove_file(temp_file).expect("Failed to remove file");
    }

    #[test]
    fn test_snapshot_channel_writes_to_disk() {
        let temp_file = get_temp_path();
        let manager = PersistenceManager::new(&temp_file);

        let sender = manager.start_memory_snapshot();

        let now = chrono::Utc::now().timestamp();
        let job1 = Job::new(now + 1000, 1, "Task 1", "func1").unwrap();
        let mut job2 = Job::new(now + 2000, 2, "Task 2", "func2").unwrap();
        job2.status = Status::Running;

        let snapshot = vec![job1.clone(), job2.clone()];

        sender
            .send(snapshot)
            .expect("Failed to send snapshot to channel");

        thread::sleep(Duration::from_millis(50)); // Wait for system to write the job in disk (prevent race condition)

        let loaded_jobs = manager.load_jobs();

        assert_eq!(loaded_jobs.len(), 2);

        let loaded_job1 = loaded_jobs.iter().find(|j| j.id == job1.id).unwrap();
        assert_eq!(loaded_job1.description, "Task 1");
        assert_eq!(loaded_job1.status, Status::Pending);

        let loaded_job2 = loaded_jobs.iter().find(|j| j.id == job2.id).unwrap();
        assert_eq!(loaded_job2.description, "Task 2");
        assert_eq!(loaded_job2.status, Status::Running);

        fs::remove_file(temp_file).expect("Failed to remove file");
    }
}
