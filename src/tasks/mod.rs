use std::sync::mpsc::Sender;

pub trait Task {
    fn run(log_tx: Sender<String>);
}

pub mod backup_db;
pub mod send_email;
pub mod hotfix;