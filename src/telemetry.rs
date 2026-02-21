use sysinfo::System;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

pub fn init_telemetry() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
    info!("Event System & Telemetry initialized.");
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
