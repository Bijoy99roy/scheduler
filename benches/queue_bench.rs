use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scheduler::{job::Job, queue::QueueManager};
use std::collections::BinaryHeap;

struct BaselineQueueManager {
    heap: BinaryHeap<Job>,
}

impl BaselineQueueManager {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    fn push(&mut self, job: Job) {
        self.heap.push(job);
    }

    fn update_status(&mut self, id: uuid::Uuid) -> bool {
        let mut all: Vec<Job> = self.heap.drain().collect();
        let found = all.iter_mut().find(|j| j.id == id);
        let updated = if let Some(job) = found {
            job.status = scheduler::job::Status::Running;
            true
        } else {
            false
        };
        self.heap = BinaryHeap::from(all);
        updated
    }
}

fn make_jobs(count: usize, start_time: i64) -> Vec<Job> {
    (0..count)
        .map(|i| {
            let t = start_time + i as i64;
            let p = (i % 16) as u8;
            Job::new(t, p, format!("job-{i}"), "noop", 3).expect("job time should be in the future")
        })
        .collect()
}

fn seed_current(q: &mut QueueManager, jobs: &[Job]) {
    for job in jobs.iter().cloned() {
        q.push(job);
    }
}

fn seed_baseline(q: &mut BaselineQueueManager, jobs: &[Job]) {
    for job in jobs.iter().cloned() {
        q.push(job);
    }
}

fn bench_update_status(c: &mut Criterion) {
    let mut group = c.benchmark_group("update_status_at_10k");
    println!(
        "Benchmarking 10k queue update_status: current vs baseline_upstream_update_status (goal: sub-microsecond updates)."
    );

    group.bench_function("current_update_status", |b| {
        let base = Job::now() + 1_000_000;
        let jobs = make_jobs(10_000, base);
        let target_id = jobs[5_000].id;
        let mut q = QueueManager::new();
        seed_current(&mut q, &jobs);

        b.iter(|| {
            let updated = q.update_status(target_id, scheduler::job::Status::Running);
            black_box(updated);
        });
    });

    group.bench_function("baseline_upstream_update_status", |b| {
        let base = Job::now() + 1_000_000;
        let jobs = make_jobs(10_000, base);
        let target_id = jobs[5_000].id;
        let mut q = BaselineQueueManager::new();
        seed_baseline(&mut q, &jobs);

        b.iter(|| {
            let updated = q.update_status(target_id);
            black_box(updated);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_update_status);
criterion_main!(benches);
