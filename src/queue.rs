use crate::job::{Job, Status};
use std::collections::BinaryHeap;
use uuid::Uuid;

#[derive(Default)]
pub struct QueueManager {
    heap: BinaryHeap<Job>,
}

#[allow(dead_code)]
impl QueueManager {
    pub fn new() -> Self {
        QueueManager {
            heap: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, job: Job) {
        self.heap.push(job);
    }

    pub fn pop(&mut self) -> Option<Job> {
        self.heap.pop()
    }

    pub fn remove(&mut self, id: Uuid) -> Option<Job> {
        let mut all: Vec<Job> = self.heap.drain().collect();
        let pos = all.iter().position(|j| j.id == id);
        match pos {
            Some(i) => {
                let removed = all.remove(i);
                self.heap = BinaryHeap::from(all);
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
                ready.push(self.pop().unwrap());
            } else {
                break;
            }
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
