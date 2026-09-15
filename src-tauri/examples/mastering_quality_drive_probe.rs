//! Offline C1 experiment; requested settings and production policy are unchanged.
use yes_master_lib::types;
#[path = "research/drive_probe.rs"]
mod drive_probe;
#[path = "../src/sample_rate.rs"]
mod sample_rate;

fn main() {
    drive_probe::run();
}
