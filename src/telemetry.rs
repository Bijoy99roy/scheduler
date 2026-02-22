use sysinfo::System;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt};

pub fn init_telemetry() -> tracing_appender::non_blocking::WorkerGuard {
    // 1. File Logger (rolling daily)
    let file_appender = tracing_appender::rolling::daily("logs", "scheduler.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Create layer for file output
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    // 2. Terminal Logger
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout);

    // Combine both layers
    let subscriber = tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(stdout_layer)
        .with(file_layer);

    // Set as global
    let _ = tracing::subscriber::set_global_default(subscriber);
    info!("Event System & Telemetry initialized.");

    // Return the guard so it stays alive for the duration of main
    // Note: We need to alter main.rs to capture this guard too!
    guard
}

pub fn log_resource_usage() {
    let mut sys = System::new_all();
    sys.refresh_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_all();

    let total_memory = sys.total_memory() / 1024 / 1024; // MB
    let used_memory = sys.used_memory() / 1024 / 1024; // MB
    let cpu_usage = sys.global_cpu_usage();

    info!(
        "Resource Monitor - Memory: {}MB / {}MB, CPU: {:.2}%",
        used_memory, total_memory, cpu_usage
    );
}
