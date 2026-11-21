#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|argument| argument == "--probe-sensors") {
        framelab_lib::probe::run();
        return;
    }
    framelab_lib::run();
}
