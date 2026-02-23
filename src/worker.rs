use std::collections::HashMap;
use std::sync::mpsc::Sender;

use crate::job::Job;

/// Type alias for a function pointer that takes log_tx and returns nothing
// type JobFn = fn(Sender<String>);

pub struct Worker {
    registry: HashMap<String, Box<dyn Fn(Sender<String>) + Send>>,
}

impl Worker {
    /// Initialize a new worker with an empty registry
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Register a function string to a concrete function
    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Sender<String>) + Send + 'static,
    {
        self.registry.insert(name.to_string(), Box::new(f));
    }

    /// The execution engine: looks up the string in the map and calls the function
    pub fn run_job(&self, job: &mut Job, log_tx: Sender<String>) {
        if let Some(func) = self.registry.get(&job.function) {
            job.start();
            let _ = log_tx.send(format!("[Worker] Executing '{}'", job.description));
            func(log_tx.clone()); // Execute the function
            job.complete();
            let _ = log_tx.send(format!("[Worker] Done '{}'", job.description));
        } else {
            let _ = log_tx.send(format!(
                "[Worker] Error: No function registered for '{}'",
                job.function
            ));
            job.fail_and_retry();
        }
    }

    /// Starts a simple blocking loop to process jobs from the channel
    pub fn start(&self, rx: std::sync::mpsc::Receiver<Job>, log_tx: Sender<String>) {
        for mut job in rx {
            self.run_job(&mut job, log_tx.clone());
        }
    }
}

// --- Task Functions ---

pub fn send_email(log_tx: Sender<String>) {
    let _ = log_tx.send("📧 [Task] Sending email...".to_string());

    let api_key = std::env::var("RESEND_API_KEY").unwrap_or_default();
    let from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "onboarding@resend.dev".to_string());
    let to = std::env::var("SMTP_RECIPIENT").unwrap_or_default();

    if api_key.is_empty() {
        let _ = log_tx.send("❌ [Task] Error: RESEND_API_KEY missing in .env!".to_string());
        return;
    }
    if to.is_empty() {
        let _ = log_tx.send("❌ [Task] Error: SMTP_RECIPIENT missing in .env!".to_string());
        return;
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    let body_text = format!(
        "Hello!\n\nThe automated email task has been successfully processed by your Termi-Schedule worker thread.\n\nTimestamp: {}",
        timestamp
    );

    let client = reqwest::blocking::Client::new();
    let body = serde_json::json!({
        "from": from,
        "to": [to],
        "subject": "Termi-Schedule: Job Executed ✅",
        "text": body_text,
    });

    match client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
    {
        Ok(resp) => {
            if resp.status().is_success() {
                let _ = log_tx.send("✅ [Task] Email sent successfully!".to_string());
            } else {
                let status = resp.status();
                let text = resp.text().unwrap_or_default();
                let _ = log_tx.send(format!("❌ [Task] Resend API error ({}): {}", status, text));
            }
        }
        Err(e) => {
            let _ = log_tx.send(format!("❌ [Task] HTTP request failed: {}", e));
        }
    }
}

pub fn backup_db(log_tx: Sender<String>) {
    let _ = log_tx.send("🗄️ [Task] Backing up database...".to_string());
    // Logic for DB backup here
}
