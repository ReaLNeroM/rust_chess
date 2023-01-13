extern crate sfml;

pub mod action;
pub mod actions;
pub mod ai;
pub mod result;
pub mod state;
pub mod ui;
pub mod value;

use state::PC;
use std::collections::HashMap;
use std::process;
use ui::{ui_routine, Thinker};

fn main() {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            let pid_info = process::id();
            out.finish(format_args!(
                "[{}][{}][{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                pid_info,
                record.target(),
                record.level(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("hyper", log::LevelFilter::Info)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        .chain(fern::log_file("chess.log").unwrap())
        // Apply globally
        .apply()
        .unwrap();

    let mut color_assignments = HashMap::new();
    color_assignments.insert(PC::White, Thinker::Player);
    color_assignments.insert(PC::Black, Thinker::AI);

    ui_routine(color_assignments);
}
