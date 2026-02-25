use crate::job::{Job, Status};
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use std::collections::HashMap;
use uuid::Uuid;

type JobPriority = (Reverse<i64>, u8);

#[derive(Default)]
pub struct QueueManager {
    heap: PriorityQueue<Uuid, JobPriority>,
    jobs: HashMap<Uuid, Job>,
    snapshot_tx: Option<std::sync::mpsc::Sender<Vec<Job>>>,
}

#[allow(dead_code)]
impl QueueManager {
    pub fn new() -> Self {
        QueueManager {
            heap: PriorityQueue::new(),
            jobs: HashMap::new(),
            snapshot_tx: None,
        }
    }

    pub fn set_persistence(&mut self, tx: std::sync::mpsc::Sender<Vec<Job>>) {
        self.snapshot_tx = Some(tx);
    }

    pub fn load_from_vec(&mut self, jobs: Vec<Job>) {
        for job in jobs {
            let priority = (Reverse(job.execution_time), job.priority);
            let id = job.id;
            self.jobs.insert(id, job);
            self.heap.push(id, priority);
        }

        self.notify_persistence();
    }

    fn notify_persistence(&mut self) {
        if let Some(tx) = self.snapshot_tx.as_ref() {
            let snapshot = self.snapshot();
            let _ = tx.send(snapshot);
        }
    }

    pub fn push(&mut self, job: Job) {
        let priority = (Reverse(job.execution_time), job.priority);
        let id = job.id;
        self.jobs.insert(id, job);
        self.heap.push(id, priority);
        self.notify_persistence();
    }

    pub fn pop(&mut self) -> Option<Job> {
        let result = self.heap.pop().and_then(|(id, _)| self.jobs.remove(&id));
        if result.is_some() {
            self.notify_persistence();
        }
        result
    }

    pub fn remove(&mut self, id: Uuid) -> Option<Job> {
        match self.heap.remove(&id) {
            Some(_) => {
                let removed = self.jobs.remove(&id);
                self.notify_persistence();
                removed
            }
            None => None,
        }
    }

    pub fn peek(&self) -> Option<&Job> {
        self.heap.peek().and_then(|(id, _)| self.jobs.get(id))
    }

    pub fn pop_ready(&mut self, now: i64) -> Vec<Job> {
        let mut ready = Vec::new();
        while let Some((id, _)) = self.heap.peek() {
            // We use the peeked ID to check the actual job time in the HashMap
            if let Some(job) = self.jobs.get(id) {
                if job.execution_time <= now {
                    let (removed_id, _) = self.heap.pop().unwrap();
                    ready.push(self.jobs.remove(&removed_id).unwrap());
                } else {
                    break;
                }
            }
        }

        if !ready.is_empty() {
            self.notify_persistence();
        }
        ready
    }

    pub fn update_status(&mut self, id: Uuid, new_status: Status) -> bool {
        if let Some(job) = self.jobs.get_mut(&id) {
            job.status = new_status;
            self.notify_persistence();
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Returns all jobs sorted by execution time (for display or persistence)
    pub fn snapshot(&self) -> Vec<Job> {
        let mut v: Vec<Job> = self.jobs.values().cloned().collect();
        v.sort_by(|a, b| a.execution_time.cmp(&b.execution_time).then(b.priority.cmp(&a.priority)));
        v
    }
}