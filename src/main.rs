#![feature(map_first_last, clamp, let_chains)]

#[macro_use]
extern crate pest_derive;
extern crate pest;

#[macro_use]
mod utils;
mod model;
mod app;
mod ui;

fn print_usage(args: Vec<String>) {
    eprintln!("Usage: {} path_to_profile", args[0]);
}

fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("PhaseViz error: No path to profile specified");
        print_usage(args);
        std::process::exit(1);
    }

    let profile_path = &args[1];

    let profile = model::profile::Profile::parse(profile_path);
    let synced_profile = profile.synced();
    let mut app = app::App::new(&synced_profile);
    
    app.run();
}