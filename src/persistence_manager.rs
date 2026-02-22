use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread,
};

use crate::job::Job;

pub struct PersistenceManager {
    pub storage_path: PathBuf,
}

impl PersistenceManager {
    pub fn new(storage_path: &str) -> Self {
        Self {
            storage_path: PathBuf::from(storage_path),
        }
    }

    pub fn start_memory_snapshot(&self) -> Sender<Vec<Job>> {
        let (tx, rx) = mpsc::channel();
        let path = self.storage_path.clone();

        thread::spawn(move || {
            println!("System will snapshot the indexed jobs...");
            for jobs_snapshot in rx {
                match serde_json::to_string_pretty(&jobs_snapshot) {
                    Ok(json) => {
                        let temp_path = path.with_extension("tmp");
                        if let Err(e) = fs::write(&temp_path, &json) {
                            eprintln!("Error: Failed to write to temp disk: {}", e);
                            continue;
                        }
                        if let Err(e) = fs::rename(&temp_path, &path) {
                            eprintln!("Error: Failed to swap persistent file: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to serialize job: {}", e),
                }
            }
        });
        tx
    }

    pub fn load_jobs(&self) -> Vec<Job> {
        let Ok(data) = fs::read_to_string(&self.storage_path) else {
            return Vec::new();
        };

        let Ok(jobs) = serde_json::from_str::<Vec<Job>>(&data) else {
            eprintln!("Failed to parse json");
            return Vec::new();
        };

        println!("Successfully loaded {} jobs from disk", jobs.len());
        jobs
    }
}
