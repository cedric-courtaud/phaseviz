#![feature(map_first_last, clamp, let_chains)]

#[macro_use]
extern crate pest_derive;
extern crate pest;

#[macro_use]
mod utils;
mod profile;
mod parser;
mod app;
mod ui;

fn main() {
    let mut profile = profile::Profile::parse("assets/test/memviz.chekpoint.8446");
    let synced_profile = profile.synced();
    let mut app = app::App::new(&synced_profile);
    app.run();
}