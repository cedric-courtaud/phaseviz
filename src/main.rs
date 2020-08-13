#![feature(map_first_last)]
use std::collections::BTreeSet;

#[macro_use]
extern crate pest_derive;
extern crate pest;

mod profile;
mod parser;
mod sourcefile;
// mod ui;

// use tui::backend::TermionBackend;
// use std::io::{stdout};
// use termion::raw::IntoRawMode;

fn main() {
    // ui::PhasevizTUI::new(&profile::Profile::parse("assets/test/memviz.chekpoint.28516")).run();
}