use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub execution_time: i64,
    pub priority: u8,
    pub description: String,
    pub function: String,
    pub status: Status,
    pub max_retries: u32,
    pub retry_count: u32,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

impl Job {
    pub fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    pub fn new(
        execution_time: i64,
        priority: u8,
        description: impl Into<String>,
        function: impl Into<String>,
        max_retries: u32,
    ) -> Result<Job, String> {
        if execution_time < Self::now() {
            return Err(format!("execution_time {} is in the past", execution_time));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            execution_time,
            priority,
            description: description.into(),
            function: function.into(),
            status: Status::Pending,
            max_retries,
            retry_count: 0,
            metadata: None,
        })
    }

    pub fn start(&mut self) {
        self.status = Status::Running;
        info!("Job {} started running.", self.id);
    }

    pub fn complete(&mut self) {
        self.status = Status::Success;
        info!("Job {} completed successfully.", self.id);
    }

    pub fn fail_and_retry(&mut self) -> bool {
        if self.retry_count < self.max_retries {
            self.retry_count += 1;
            self.status = Status::Pending;
            warn!(
                "Job {} failed. Retrying ({}/{}).",
                self.id, self.retry_count, self.max_retries
            );
            true
        } else {
            self.status = Status::Failed;
            warn!(
                "Job {} failed permanently after {} retries.",
                self.id, self.max_retries
            );
            false
        }
    }
}

impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Job {}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Job {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .execution_time
            .cmp(&self.execution_time)
            .then(self.priority.cmp(&other.priority))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_state_transitions() {
        let mut job = Job::new(Job::now() + 10, 1, "desc", "func", 2).unwrap();
        assert_eq!(job.status, Status::Pending);

        job.start();
        assert_eq!(job.status, Status::Running);

        job.complete();
        assert_eq!(job.status, Status::Success);
    }

    #[test]
    fn test_job_retries() {
        let mut job = Job::new(Job::now() + 10, 1, "desc", "func", 1).unwrap();

        // Fail once - should retry
        let can_retry = job.fail_and_retry();
        assert!(can_retry);
        assert_eq!(job.retry_count, 1);
        assert_eq!(job.status, Status::Pending);

        // Fail twice - should exceed max_retries and fail
        let can_retry_again = job.fail_and_retry();
        assert!(!can_retry_again);
        assert_eq!(job.retry_count, 1);
        assert_eq!(job.status, Status::Failed);
    }
}
