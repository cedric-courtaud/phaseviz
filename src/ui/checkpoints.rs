use crate::model::profile::ProfileItem;

use tui::{
    backend::Backend,
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::Paragraph,
    Frame
};

use super::{help_widget, Panel, PanelBox};

use std::collections::btree_set::BTreeSet;

fn get_checkpoints<'a, T: AsRef<[&'a ProfileItem]>>(items: T) -> Vec<u32> {
    let mut checkpoints = BTreeSet::new();

    for item in items.as_ref() {
        match item {
            ProfileItem::File(f) => {
                for c in f.borrow().checkpoints.iter() {
                    checkpoints.insert(*c);
                }
            }
            ProfileItem::Line(_, l) => {
                for c in l.checkpoints.iter() {
                    checkpoints.insert(*c);
                }
            }
        }
    }

    checkpoints.iter().cloned().collect()
}

fn number_of_digits(number: u32) -> usize {
    let mut ret = 0;
    let mut n = number;

    while n > 0 {
        n /= ((ret + 1) * 10) as u32;
        ret += 1;
    }

    ret
}

fn format_header_cell<'a>(id: u32, cell_width: usize) -> Span<'a> {
    Span::raw(format!("{:^1$}", id, cell_width))
}

fn format_cell<'a>(met: bool, cell_width: usize) -> Span<'a> {
    // let status_char = if met {"◼︎"} else {"◻︎"};
    let status_char = if met { '◼' } else { '·' };

    let style = if met {
        Style::default().fg(Color::LightGreen)
    } else {
        Style::default().fg(Color::Gray)
    };

    Span::styled(format!("{:^1$}", status_char, cell_width), style)
}

fn checkpoints_header<'a>(checkpoints: &Vec<u32>, cell_width: usize) -> Spans<'a> {
    let mut spans = vec![];

    if checkpoints.len() > 0 {
        for checkpoint in checkpoints {
            spans.push(format_header_cell(*checkpoint, cell_width));
        }
    }

    Spans::from(spans)
}

fn checkpoints_line<'a>(
    item: &'a ProfileItem,
    checkpoints: &Vec<u32>,
    cell_width: u16,
) -> Spans<'a> {
    let mut spans = vec![];

    for checkpoint in checkpoints {
        let met = match item {
            ProfileItem::File(f) => {
                f.borrow().checkpoints.iter().find(|c| *c == checkpoint) != None
            }
            ProfileItem::Line(_, l) => l.checkpoints.iter().find(|c| *c == checkpoint) != None,
        };

        spans.push(format_cell(met, cell_width as usize));
    }

    Spans::from(spans)
}

pub struct CheckpointPanelContext<'a> {
    pbox: PanelBox<'a>,
    cell_width: u16,
    checkpoints: Vec<u32>,
}

pub struct CheckpointPanel<'a> {
    help: Vec<(&'a str, &'a str)>,
}

impl<'a> CheckpointPanel<'a> {
    pub fn new(help: Vec<(&'a str, &'a str)>) -> CheckpointPanel<'a> {
        CheckpointPanel { help: help }
    }
}

impl<'a> Panel<'a> for CheckpointPanel<'a> {
    type Context = CheckpointPanelContext<'a>;

    fn get_context<I>(&'a self, items: I, p: PanelBox<'a>) -> Self::Context
    where
        I: AsRef<[&'a ProfileItem]>,
    {
        let checkpoints = get_checkpoints(&items);
        let max_id = if let Some(id) = checkpoints.last() {
            *id
        } else {
            0
        };
        let max_digits = number_of_digits(max_id);

        let cell_min_width = max_digits + 2;
        let width = p.body.block.inner(p.body.rect).width;

        let cell_width = if checkpoints.len() > 0 {
            usize::max(cell_min_width, width as usize / checkpoints.len()) as u16
        } else {
            0
        };

        CheckpointPanelContext {
            pbox: p,
            cell_width: cell_width,
            checkpoints: checkpoints,
        }
    }

    fn render_header<B, I>(&'a self, f: &mut Frame<B>, items: I, ctx: &Self::Context)
    where
        B: Backend,
        I: AsRef<[&'a ProfileItem]>,
    {
        let checkpoints = get_checkpoints(items);
        let header_line = checkpoints_header(&checkpoints, ctx.cell_width as usize);
        let p = Paragraph::new(Text::from(header_line)).block(ctx.pbox.header.block.clone());

        f.render_widget(p, ctx.pbox.header.rect);
    }

    fn render_body<B, I>(&'a self, f: &mut Frame<B>, items: I, ctx: &Self::Context)
    where
        B: Backend,
        I: AsRef<[&'a ProfileItem]>,
    {
        let mut checkpoint_lines: Vec<Spans> = vec![];

        for item in items.as_ref().clone() {
            checkpoint_lines.push(checkpoints_line(item, &ctx.checkpoints, ctx.cell_width));
        }

        let p = Paragraph::new(Text::from(checkpoint_lines)).block(ctx.pbox.body.block.clone());
        f.render_widget(p, ctx.pbox.body.rect);
    }

    fn render_help<B, I>(&'a self, f: &mut Frame<B>, _items: I, ctx: &Self::Context)
    where
        B: Backend,
        I: AsRef<[&'a ProfileItem]>,
    {
        let w = help_widget(&self.help).block(ctx.pbox.footer.block.clone());

        f.render_widget(w, ctx.pbox.footer.rect);
    }
}
