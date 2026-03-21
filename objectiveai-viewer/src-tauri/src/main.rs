#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

fn main() {
    let args = objectiveai_viewer::args::Args::parse();
    objectiveai_viewer::run(args)
}
