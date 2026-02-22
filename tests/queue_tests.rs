use scheduler::{job::Job, queue::QueueManager};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn job(exec_time: i64, priority: u8, desc: &str) -> Job {
    Job::new(exec_time, priority, desc, "fn").unwrap()
}

#[test]
fn push_increases_len() {
    let mut q = QueueManager::new();
    q.push(job(now() + 10, 1, "a"));
    q.push(job(now() + 20, 1, "b"));
    assert_eq!(q.len(), 2);
}

#[test]
fn pop_returns_earliest() {
    let mut q = QueueManager::new();
    q.push(job(now() + 30, 1, "last"));
    q.push(job(now() + 10, 1, "first"));
    q.push(job(now() + 20, 1, "middle"));
    assert_eq!(q.pop().unwrap().description, "first");
    assert_eq!(q.pop().unwrap().description, "middle");
    assert_eq!(q.pop().unwrap().description, "last");
}

#[test]
fn priority_breaks_time_tie() {
    let mut q = QueueManager::new();
    let t = now() + 10;
    q.push(job(t, 1, "low"));
    q.push(job(t, 9, "high"));
    assert_eq!(q.pop().unwrap().description, "high");
}

#[test]
fn peek_does_not_remove() {
    let mut q = QueueManager::new();
    q.push(job(now() + 10, 1, "only"));
    assert_eq!(q.peek().unwrap().description, "only");
    assert_eq!(q.len(), 1);
}

#[test]
fn remove_by_uuid() {
    let mut q = QueueManager::new();
    let j = job(now() + 10, 1, "target");
    let id = j.id;
    q.push(j);
    q.push(job(now() + 20, 1, "other"));
    assert!(q.remove(id).is_some());
    assert_eq!(q.len(), 1);
}

#[test]
fn remove_missing_uuid_returns_none() {
    let mut q = QueueManager::new();
    q.push(job(now() + 10, 1, "job"));
    assert!(q.remove(Uuid::new_v4()).is_none());
    assert_eq!(q.len(), 1);
}

#[test]
fn pop_ready_only_returns_due_jobs() {
    let mut q = QueueManager::new();
    let base = now();
    q.push(job(base + 10, 1, "first ready"));
    q.push(job(base + 20, 1, "second ready"));
    q.push(job(base + 999, 1, "not ready"));
    let ready = q.pop_ready(base + 20);
    assert_eq!(ready.len(), 2);
    assert_eq!(q.len(), 1);
}

#[test]
fn pop_on_empty_returns_none() {
    let mut q = QueueManager::new();
    assert!(q.pop().is_none());
}

#[test]
fn rejects_job_with_past_execution_time() {
    let result = Job::new(0, 5, "old job", "fn");
    assert!(result.is_err());
}

#[test]
fn accepts_job_with_future_execution_time() {
    let result = Job::new(now() + 100, 5, "future job", "fn");
    assert!(result.is_ok());
}
