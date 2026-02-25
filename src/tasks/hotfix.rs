use std::sync::mpsc::Sender;
use super::Task;

pub struct HotfixTask;

impl Task for HotfixTask {
    fn run(log_tx: Sender<String>) {
        let _ = log_tx.send(" [Task] Applying urgent hotfix...".to_string());
    }
}