# Termi-Schedule ⏱️

A high-performance, asynchronous, time-based priority task scheduler built completely in Rust. This project features a completely decoupled architecture, offering advanced queue management, concurrent worker execution, crash-safe persistence, and an interactive Terminal User Interface (TUI).

## 🚀 Key Features

- **Time & Priority Execution Engine**: Utilizes a customized highly optimized Min-Heap data structure to efficiently order and process jobs based on accurate execution UNIX timestamps and dynamic priority levels.
- **Decoupled Worker Threads**: Employs scalable, non-blocking asynchronous loops utilizing multi-producer, single-consumer (mpsc) message channels to dispatch tasks securely from the queue to the worker pool.
- **Crash-Safe Persistence**: Features background persistence memory that asynchronously saves the queue state to disk (`queue.json`), guaranteeing zero data loss upon application restart.
- **Telemetry & Retries**: Includes robust execution retries and dedicated asynchronous file-based tracing telemetry (`scheduler.log`), achieving high observability thread-safely via `tracing_appender::non_blocking`.
- **Interactive TUI Dashboard**: Features a sleek, responsive live terminal dashboard using `ratatui` configured with `crossterm`. Features include live queue snapshots, scrolling worker logs, and interactive "Add Task" forms without blocking the main engine.

## 📦 Project Structure

```bash
scheduler/
├── Cargo.toml
└── src/
    ├── main.rs                 # Initializes systems & spins up the TUI/Engine/Workers.
    ├── engine.rs               # TimePriorityEngine: Polls the queue logically and dispatches.
    ├── worker.rs               # The execution engine: Looks up mapped functions & runs logic.
    ├── queue.rs                # Core min-heap logic with thread-safe Arc<Mutex<>> wrappings.
    ├── job.rs                  # Core Job datastructure featuring execution states & retries.
    ├── persistence_manager.rs  # Background thread gracefully syncing memory queue to disk.
    ├── telemetry.rs            # Non-blocking file/terminal logging integration.
    └── tui.rs                  # The core interactive ratatui layout and listener state.
```

## 🛠️ Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)

### Build & Run

```bash
cargo build --release
```

Run the interactive visual interface out of the box:

```bash
cargo run
```

### Usage Controls (TUI)

- **`Ctrl+A`**: Add a new job (Opens input modal).
- **`D` / `Delete`**: Delete the currently selected task from the active queue.
- **`↑` / `↓`**: Navigate the task list.
- **`Enter`**: Submit Job parameters.
- **`Q`, `Esc`, or `Ctrl+C`**: Gracefully shutdown the engine, preserve the exact state to disk, and quit.

## 🧪 Testing

The repository contains extensive unit tests ensuring thread safety, queue consistency, and graceful engine handling.

Run the test suite via:

```bash
cargo test
```
