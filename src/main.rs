#![feature(map_first_last, clamp)]
use std::collections::BTreeSet;
use std::usize;

#[macro_use]
extern crate pest_derive;
extern crate pest;

mod profile;
mod parser;
mod sourcefile;
mod app;
mod ui;

use tui::backend::TermionBackend;
use std::io::{stdout};
use termion::raw::IntoRawMode;

fn main() {
    let mut profile = profile::Profile::parse("assets/test/memviz.chekpoint.28516");
    profile.sync_with_fs();
    let mut app = app::App::new(&profile);
    app.run();
    println!("{:?}", profile);
}