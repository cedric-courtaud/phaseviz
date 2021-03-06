use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::controller::App;
use crate::model::profile::ProfileItem;

mod addr_range;
mod checkpoints;
mod source;

pub fn help_widget<'a, T: AsRef<[(&'a str, &'a str)]>>(items: T) -> Paragraph<'a> {
    let block = Block::default()
        .borders(Borders::TOP)
        .style(Style::default());
    let mut spans: Vec<Span> = vec![Span::from("")];

    for i in items.as_ref() {
        spans.push(Span::styled(
            format!("[{}] ", i.0),
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::from(i.1));
    }

    Paragraph::new(Spans::from(spans)).block(block)
}

pub struct PanelPart<'a> {
    rect: Rect,
    block: Block<'a>,
}

pub struct PanelBox<'a> {
    header: PanelPart<'a>,
    body: PanelPart<'a>,
    footer: PanelPart<'a>,
}

pub trait Panel<'a> {
    type Context;

    fn get_context<I>(&'a self, items: I, p: PanelBox<'a>) -> Self::Context
    where
        I: AsRef<[&'a ProfileItem]>;
    fn render_header<B, I>(&'a self, f: &mut Frame<B>, items: I, ctx: &Self::Context)
    where
        B: Backend,
        I: AsRef<[&'a ProfileItem]>;
    fn render_body<B, I>(&'a self, f: &mut Frame<B>, items: I, ctx: &Self::Context)
    where
        B: Backend,
        I: AsRef<[&'a ProfileItem]>;
    fn render_help<B, I>(&'a self, f: &mut Frame<B>, items: I, ctx: &Self::Context)
    where
        B: Backend,
        I: AsRef<[&'a ProfileItem]>;
}

pub fn render_panel<'a, P, B, T>(p: &'a mut P, f: &mut Frame<B>, rect: Rect, items: T)
where
    P: Panel<'a>,
    B: tui::backend::Backend,
    T: AsRef<[&'a ProfileItem]>,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(rect);

    let header_chunk = chunks[0];
    let main_chunk = chunks[1];
    let help_chunk = chunks[2];

    let main_block = Block::default().borders(Borders::NONE);
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .style(Style::default());
    let help_block = Block::default().borders(Borders::TOP);

    let panel_box = PanelBox {
        header: PanelPart {
            rect: header_chunk,
            block: header_block,
        },
        body: PanelPart {
            rect: main_chunk,
            block: main_block,
        },
        footer: PanelPart {
            rect: help_chunk,
            block: help_block,
        },
    };

    let ctx = p.get_context(&items, panel_box);

    p.render_body(f, &items, &ctx);
    p.render_header(f, &items, &ctx);
    p.render_help(f, &items, &ctx);
}

pub fn draw<B: tui::backend::Backend>(f: &mut Frame<B>, app: &mut App) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(0),
                Constraint::Min(0),
                Constraint::Length(0),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(f.size());

    let header_chunk = vertical_chunks[0];
    let main_chunk = vertical_chunks[1];
    let footer_chunk = vertical_chunks[2];

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Min(0),
                Constraint::Length(30),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(main_chunk);

    let _header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(header_chunk);

    let _footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(0)
        .split(footer_chunk);

    let checkpoints_chunk = main_chunks[0];
    let addr_chunk = main_chunks[1];
    let source_chunk = main_chunks[2];

    let source_block = Block::default().borders(Borders::ALL);

    let h = source_block.inner(source_chunk).height;
    app.set_height(h - 4);

    let a = app.get_y_pos();
    let b = std::cmp::min(a + (h as usize), app.items.len() - 1);

    let items = &app.items[a..=b];

    // let t2 = "Placeholder";
    // let info_text = Text::from(t2);

    /*
    let info_chunk = footer_chunks[0];
    let info_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Red));

    let info = Paragraph::new(info_text).block(info_block);
    */

    let source_outter_block = Block::default().borders(Borders::BOTTOM | Borders::TOP);
    f.render_widget(source_outter_block, source_chunk);
    
    let addr_outter_block = Block::default().borders(Borders::BOTTOM | Borders::TOP);
    f.render_widget(addr_outter_block, addr_chunk);

    let checkpoints_outter_block = Block::default().borders(Borders::BOTTOM | Borders::TOP);
    f.render_widget(checkpoints_outter_block, checkpoints_chunk);

    let mut checkpoint_panel = checkpoints::CheckpointPanel::new(vec![("H", "Help")]);
    let mut addr_panel = addr_range::InstAddrPanel::new(vec![("H", "Help")]);
    let mut source_panel = source::SourcePanel::new(vec![("H", "Help")]);

    render_panel(&mut checkpoint_panel, f, checkpoints_chunk, &items);
    render_panel(&mut addr_panel, f, addr_chunk, &items);
    render_panel(&mut source_panel, f, source_chunk, &items);
}
