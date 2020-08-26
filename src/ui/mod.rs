use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    text::{Text, Span, Spans},
    Frame,
    backend::Backend,
    style::{Style, Color, Modifier}
};

use crate::app::App;
use crate::app::ProfileItem;

mod checkpoints;
mod source;

pub fn help_widget<'a, T: AsRef<[(&'a str, &'a str)]>>(items: T) -> Paragraph<'a> {
        let block   = Block::default().borders(Borders::TOP).style(Style::default());
        let mut spans: Vec<Span> = vec!(Span::from(""));

        for i in items.as_ref() {
            spans.push(Span::styled(format!("[{}] ", i.0), Style::default().add_modifier(Modifier::BOLD)));
            spans.push(Span::from(i.1));
        }

        Paragraph::new(Spans::from(spans)).block(block)
}

pub trait Panel<'a> {
    fn render_header<B, I>(&'a self,f: &mut Frame<B>, items: I, rect: Rect, block: Block) where B: Backend, I: AsRef<[ProfileItem<'a>]>;
    fn render_body<B, I>(&'a self,f: &mut Frame<B>, items: I, rect: Rect, block: Block) where B: Backend, I: AsRef<[ProfileItem<'a>]>;
    fn render_help<B>(&'a self,f: &mut Frame<B>, rect: Rect, block: Block) where B: Backend;
}

pub fn render_panel<'a, P, B, T>(p: &'a P, f: &mut Frame<B>, rect: Rect, items: T) 
where P: Panel<'a>, B: tui::backend::Backend,T: AsRef<[ProfileItem<'a>]> {
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

        let main_block   = Block::default().borders(Borders::NONE);
        let header_block = Block::default().borders(Borders::BOTTOM).style(Style::default());
        let help_block = Block::default().borders(Borders::TOP);
        
        p.render_body(f, &items, main_chunk, main_block);
        p.render_header(f, &items, header_chunk, header_block);
        p.render_help(f, help_chunk, help_block);
}

pub fn draw<B: tui::backend::Backend>(f: &mut Frame<B>, app: &mut App) {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
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
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ].as_ref())
            .margin(1)
            .split(main_chunk);
        
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(100),
            ].as_ref())
            .margin(0)
            .split(header_chunk);
        
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(100),
            ].as_ref())
            .margin(0)
            .split(footer_chunk);

        let checkpoints_chunk = main_chunks[0];
        let source_chunk = main_chunks[1];

        let info_chunk = footer_chunks[0];


        let source_block = Block::default().borders(Borders::ALL);

        let h = source_block.inner(source_chunk).height;
        app.set_height(h - 4);
        
        let a = app.get_y_pos();
        let b = std::cmp::min(a + (h as usize), app.items.len() - 1);

        let items = &app.items[a..=b];

        let t2 = "Placeholder";
        let info_text = Text::from(t2);

        let info_block = Block::default().borders(Borders::ALL)
                                         .style(Style::default().bg(Color::Red));

        let info = Paragraph::new(info_text).block(info_block);
        
        let source_outter_block = Block::default().borders(Borders::BOTTOM | Borders::TOP);
        f.render_widget(source_outter_block, source_chunk);
        
        let checkpoints_outter_block = Block::default().borders(Borders::BOTTOM | Borders::TOP);
        f.render_widget(checkpoints_outter_block, checkpoints_chunk);

        let checkpoint_panel = checkpoints::CheckpointPanel::new(vec!(("H", "Help")));
        let source_panel = source::SourcePanel::new(vec!(("H", "Help")));

        render_panel(&checkpoint_panel, f, checkpoints_chunk, items);
        render_panel(&source_panel, f, source_chunk, items);
}