use crate::app::{ProfileItem};

use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    text::{Text, Span, Spans},
    backend::Backend,
    Frame,
    style::{Style, Modifier}
};

use super::{Panel, help_widget};

use std::collections::btree_set::BTreeSet;

fn get_checkpoints <'a,T: AsRef<[ProfileItem<'a>]>>(items: T) -> Vec<u32> {
    let mut checkpoints = BTreeSet::new();
    
    for item in items.as_ref() {
        match item {
             ProfileItem::FileHeader(f) => f.get_checkpoints().iter().for_each(|c| {checkpoints.insert(*c);}),
             ProfileItem::CodeLine(l) => l.checkpoints.iter().for_each(|c| {checkpoints.insert(*c);}),
             ProfileItem::FunctionLine(l) => l.checkpoints.iter().for_each(|c| {checkpoints.insert(*c);}),
        }
    }

    checkpoints.iter().cloned().collect()
}

fn number_of_digits(number: u32) -> usize {
    let mut ret = 0;
    let mut n = number;

    while n > 0 {
        n   /= ((ret + 1) * 10) as u32;
        ret += 1;
    }

    ret
}

fn format_header_cell(id: u32, cell_width: usize) -> String {
    format!("{:^1$}", id, cell_width)
}

fn format_cell<'a>(met: bool, cell_width: usize) -> Span<'a> {
    let status_char = if met {"◼︎"} else {"◻︎"};

    Span::raw(format!("{:^1$}", status_char, cell_width))
}

fn checkpoints_header(checkpoints: &Vec<u32>, width: u16) -> String {
    let mut ret = String::from("");

    if checkpoints.len() > 0 {
        let max_id = checkpoints.last().unwrap();
        let max_digits = number_of_digits(*max_id);
        let cell_width = width as usize / (max_digits + 2);

        for checkpoint in checkpoints {
            ret.push_str(&format_header_cell(*checkpoint, cell_width));
        }

    }

    ret
}

fn render_checkpoints<'a>(item: &'a ProfileItem, checkpoints: &Vec<u32>, cell_width: u16) -> Spans<'a> {
    let mut spans = vec!();

    for checkpoint in checkpoints {
        let met = match item {
            ProfileItem::FileHeader(f) => {
                f.get_checkpoints().contains(checkpoint)
            }

            ProfileItem::FunctionLine(l) | ProfileItem::CodeLine(l) => {
                l.checkpoints.contains(checkpoint)
            }
        };

        spans.push(format_cell(met, cell_width as usize));
    }

    Spans::from(spans)
}

pub struct CheckpointPanel<'a> {
    help: Vec<(&'a str, &'a str)>
}

impl<'a> CheckpointPanel<'a> {
    pub fn new(help: Vec<(&'a str, &'a str)>) -> CheckpointPanel<'a> {
        CheckpointPanel{
            help: help
        }
    }
}

impl<'a> Panel<'a> for CheckpointPanel<'a> {
    fn render_header<B, I>(&'a self,f: &mut Frame<B>, items: I, rect: Rect, block: Block) where B: Backend, I: AsRef<[ProfileItem<'a>]>{
        let checkpoints = get_checkpoints(items);
        let checkpoints_str = checkpoints_header(&checkpoints, block.inner(rect).width);
        let p = Paragraph::new(Text::from(checkpoints_str.as_str())).block(block);

        f.render_widget(p, rect);
    }

    fn render_body<B, I>(&'a self,f: &mut Frame<B>, items: I, rect: Rect, block: Block) where B: Backend, I: AsRef<[ProfileItem<'a>]>{
        let mut checkpoint_lines:Vec<Spans> = vec!();
        
        let checkpoints = get_checkpoints(&items);
        let max_id = checkpoints.last().unwrap();
        let max_digits = number_of_digits(*max_id);

        let width = block.inner(rect).width;
        let cell_width = width as usize / (max_digits + 2);


        for item in items.as_ref().clone() {
            checkpoint_lines.push(render_checkpoints(item, &checkpoints, cell_width as u16));
        }
        
        let p = Paragraph::new(Text::from(checkpoint_lines)).block(block);
        f.render_widget(p, rect);
    }

    fn render_help<B>(&'a self,f: &mut Frame<B>, rect: Rect, block: Block) where B: Backend {
        let w = help_widget(&self.help).block(block);
        
        f.render_widget(w, rect);
    }
}