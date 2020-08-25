use crate::app::{ProfileItem};

use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    text::{Text, Span, Spans},
    backend::Backend,
    Frame,
    style::{Style, Color, Modifier}
};

use super::{Panel, help_widget};

fn render_checkpoints<'a>(item: &'a ProfileItem) -> Spans<'a> {
    match item {
        ProfileItem::FileHeader(_) => {
            return Spans::from(vec![
                Span::raw("CHECKPOINT FILE PLACEHOLDER")
            ])
        }

        ProfileItem::FunctionLine(l) => {
            return Spans::from(vec![
                Span::raw(format!("{:?}", l.checkpoints))
            ])
        }

        ProfileItem::CodeLine(l) => {
            return Spans::from(vec![
                Span::raw(format!("{:?}", l.checkpoints))
            ])
        }
    }

}

pub fn draw_checkpoints<'a, B: tui::backend::Backend, T: AsRef<[ProfileItem<'a>]>>(f: &mut Frame<B>, rect: Rect, items: T) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(rect);

        let header_chunk = chunks[0];
        let main_chunk = chunks[1];
        let info_chunk = chunks[2];

        let checkpoint_lines:Vec<Spans> = items.as_ref()
                                               .iter()
                                               .map(|item| render_checkpoints(item))
                                               .collect();

        let main_block   = Block::default().borders(Borders::NONE);
        let header_block = Block::default().borders(Borders::BOTTOM).style(Style::default());
        let info_block   = Block::default().borders(Borders::TOP);
        
        let total = checkpoint_lines.len();
        let main = Paragraph::new(Text::from(checkpoint_lines)).block(main_block);
        let header = Paragraph::new(Text::from("PLACEHOLDER")).block(header_block);

        f.render_widget(main, main_chunk);
        f.render_widget(header, header_chunk);
        draw_help(f, info_chunk, [("H","Help")])
}




pub fn draw_help<'a, B: tui::backend::Backend, T: AsRef<[(&'a str, &'a str)]>>(f: &mut Frame<B>, rect: Rect, items: T) {
        let block   = Block::default().borders(Borders::TOP).style(Style::default());
        let mut spans: Vec<Span> = vec!(Span::from(""));

        for i in items.as_ref() {
            spans.push(Span::styled(format!("[{}] ", i.0), Style::default().add_modifier(Modifier::BOLD)));
            spans.push(Span::from(i.1));
        }

        let main = Paragraph::new(Spans::from(spans)).block(block);

        f.render_widget(main, rect);
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
    fn render_header<B, I>(&'a self,f: &mut Frame<B>, _items: I, rect: Rect, _block: Block) where B: Backend, I: AsRef<[ProfileItem<'a>]>{
        let p = Paragraph::new(Text::from("PLACEHOLDER"));
        f.render_widget(p, rect);
    }

    fn render_body<B, I>(&'a self,f: &mut Frame<B>, items: I, rect: Rect, block: Block) where B: Backend, I: AsRef<[ProfileItem<'a>]>{
        let mut checkpoint_lines:Vec<Spans> = vec!();

        for item in items.as_ref().clone() {
            checkpoint_lines.push(render_checkpoints(item));
        }
        
        let p = Paragraph::new(Text::from(checkpoint_lines)).block(block);
        f.render_widget(p, rect);
    }

    fn render_help<B>(&'a self,f: &mut Frame<B>, rect: Rect, block: Block) where B: Backend {
        let w = help_widget(&self.help).block(block);
        
        f.render_widget(w, rect);
    }
}