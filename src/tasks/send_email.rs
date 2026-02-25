use std::sync::mpsc::Sender;
use super::Task;

pub struct SendEmailTask;

impl Task for SendEmailTask {
    fn run(log_tx: Sender<String>) {
        let _ = log_tx.send("📧 [Task] Sending email...".to_string());
        // Logic for sending email here
    }
}