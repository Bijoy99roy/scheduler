use serde::{Deserialize, Serialize};

use std::time::{SystemTime, UNIX_EPOCH};

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
}

impl Job {

    fn now() -> i64 {
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
        })
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
