use crate::job::{Job, Status};
use std::collections::BinaryHeap;
use uuid::Uuid;

#[derive(Default)]
pub struct QueueManager {
    heap: BinaryHeap<Job>,
    db_path: String,
}

#[allow(dead_code)]
impl QueueManager {
    pub fn new() -> Self {
        Self::with_path("queue.json")
    }

    pub fn with_path(path: &str) -> Self {
        let mut heap = BinaryHeap::new();
        if !path.is_empty() {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(jobs) = serde_json::from_str::<Vec<Job>>(&data) {
                    heap = BinaryHeap::from(jobs);
                    println!("Loaded {} jobs from {}", heap.len(), path);
                }
            }
        }

        QueueManager {
            heap,
            db_path: path.to_string(),
        }
    }

    fn save_to_disk(&self) {
        if self.db_path.is_empty() { return; }

        let jobs: Vec<&Job> = self.heap.iter().collect();
        if let Ok(json) = serde_json::to_string_pretty(&jobs) {
            let _ = std::fs::write(&self.db_path, json);
        }
    }

    pub fn push(&mut self, job: Job) {
        self.heap.push(job);
        self.save_to_disk();
    }

    pub fn pop(&mut self) -> Option<Job> {
        let job = self.heap.pop();
        if job.is_some() {
            self.save_to_disk();
        }
        job
    }

    pub fn remove(&mut self, id: Uuid) -> Option<Job> {
        let mut all: Vec<Job> = self.heap.drain().collect();
        let pos = all.iter().position(|j| j.id == id);
        match pos {
            Some(i) => {
                let removed = all.remove(i);
                self.heap = BinaryHeap::from(all);
                self.save_to_disk();
                Some(removed)
            }
            None => {
                self.heap = BinaryHeap::from(all);
                None
            }
        }
    }

    pub fn peek(&self) -> Option<&Job> {
        self.heap.peek()
    }

    pub fn pop_ready(&mut self, now: i64) -> Vec<Job> {
        let mut ready = Vec::new();
        while let Some(job) = self.peek() {
            if job.execution_time <= now {
                ready.push(self.heap.pop().unwrap());
            } else {
                break;
            }
        }
        if !ready.is_empty() {
             self.save_to_disk();
        }
        ready
    }

    pub fn update_status(&mut self, id: Uuid, new_status: Status) -> bool {
        let mut all: Vec<Job> = self.heap.drain().collect();
        let found = all.iter_mut().find(|j| j.id == id);
        match found {
            Some(job) => {
                job.status = new_status;
                self.heap = BinaryHeap::from(all);
                self.save_to_disk();
                true
            }
            None => {
                self.heap = BinaryHeap::from(all);
                false
            }
        }
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
