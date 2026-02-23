use crate::job::{Job, Status};
use std::collections::BinaryHeap;
use uuid::Uuid;

pub struct QueueManager {
    heap: BinaryHeap<Job>,
    snapshot_tx: Option<std::sync::mpsc::Sender<Vec<Job>>>,
}

#[allow(dead_code)]
impl QueueManager {
    pub fn new() -> Self {
        QueueManager {
            heap: BinaryHeap::new(),
            snapshot_tx: None,
        }
    }

    pub fn set_persistence(&mut self, tx: std::sync::mpsc::Sender<Vec<Job>>) {
        self.snapshot_tx = Some(tx);
    }

    pub fn load_from_vec(&mut self, jobs: Vec<Job>) {
        self.heap = BinaryHeap::from(jobs);
    }

    fn notify_persistence(&mut self) {
        if let Some(tx) = self.snapshot_tx.clone() {
            let snapshot = self.snapshot();
            let _ = tx.send(snapshot);
        }
    }

    pub fn push(&mut self, job: Job) {
        self.heap.push(job);
        self.notify_persistence();
    }

    pub fn pop(&mut self) -> Option<Job> {
        let job = self.heap.pop();
        if job.is_some() {
            self.notify_persistence();
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
                self.notify_persistence();
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
            self.notify_persistence();
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
                self.notify_persistence();
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

    /// Returns a sorted snapshot of all jobs for display.
    pub fn snapshot(&mut self) -> Vec<Job> {
        let mut v: Vec<Job> = self.heap.drain().collect();
        v.sort();
        for j in &v {
            self.heap.push(j.clone());
        }
        v
    }
}
