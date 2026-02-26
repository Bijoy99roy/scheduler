use std::sync::mpsc::Sender;
use super::Task;

pub struct SendEmailTask;

impl Task for SendEmailTask {
    fn run(log_tx: Sender<String>) {
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

        let subject = std::env::var("EMAIL_SUBJECT")
            .unwrap_or_else(|_| "Termi-Schedule: Job Executed ✅".to_string());

        let timestamp = chrono::Utc::now().to_rfc3339();
        let default_body = format!(
            "Hello!\n\nThe automated email task has been successfully processed by your Termi-Schedule worker thread.\n\nTimestamp: {}",
            timestamp
        );
        let body_text = std::env::var("EMAIL_BODY").unwrap_or(default_body);

        let client = reqwest::blocking::Client::new();
        let body = serde_json::json!({
            "from": from,
            "to": [to],
            "subject": subject,
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
}