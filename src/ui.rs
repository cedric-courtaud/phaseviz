use tui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    text::{Text, Span, Spans},
    Frame,
    style::{Style, Color, Modifier}
};

use crate::app::App;
use crate::app::ProfileItem;

fn render_checkpoints<'a>(item: &'a ProfileItem) -> Spans<'a> {
    match item {
        ProfileItem::FileHeader(_) => {
            return Spans::from(vec![
                Span::raw("CHECKPOINT FILE PLACEHOLDER")
            ])
        }

        ProfileItem::FunctionLine(_) => {
            return Spans::from(vec![
                Span::raw("CHECKPOINT FUNCTION PLACEHOLDER")
            ])
        }

        ProfileItem::CodeLine(l) => {
            return Spans::from(vec![
                Span::raw(format!("{:?}", l.checkpoints))
            ])
        }
    }

}

fn render_code_line <'a>(item: &'a ProfileItem) -> Spans<'a> {
    match item {
        ProfileItem::FileHeader(f) => {
            return Spans::from(vec![
                Span::styled(" [fl] ", Style::default().bg(Color::Green)),
                Span::styled(format!("  {}", f.name), Style::default().add_modifier(Modifier::BOLD).add_modifier(Modifier::ITALIC))
            ])
        }

        ProfileItem::FunctionLine(_) => {
            return Spans::from(vec![

                Span::raw("CODE FUNCTION PLACEHOLDER")
            ])
        }

        ProfileItem::CodeLine(l) => {
            let line: Span;

            match l.line_content.as_ref() {
                Some(l) => line = Span::raw(format!("  {}", l)),
                None => line = Span::styled("  Source file is unavailable", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))
            }

            return Spans::from(vec![
                Span::styled(format!("{:5} ", l.nb), Style::default().bg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                line
            ])
        }
    }

}

pub fn draw<B: tui::backend::Backend>(f: &mut Frame<B>, app: &mut App) {
        let vertical_chunks = Layout::default()
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

        let checkpoints_chunk = main_chunks[0];
        let code_chunk = main_chunks[1];

        let checkpoint_block = Block::default().borders(Borders::ALL);

        let code_block = Block::default().borders(Borders::ALL);
        let h = code_block.inner(code_chunk).height;

        app.set_height(h);
        
        let a = app.get_y_pos();
        let b = std::cmp::min(a + (h as usize), app.items.len() - 1);

        let checkpoint_lines:Vec<Spans> = app.items[a..=b].iter().map(|item| render_checkpoints(item)).collect();
        let code_lines:Vec<Spans> = app.items[a..=b].iter().map(|item| render_code_line(item)).collect();

        let total = checkpoint_lines.len();
        let code = Paragraph::new(Text::from(code_lines)).block(code_block);
        
        let checkpoints = Paragraph::new(Text::from(checkpoint_lines)).block(checkpoint_block);

        let t2 = format!("({}, {}, {}, {})\n", a, b, h, total);
        let info_text = Text::from(t2.as_str());

        let info_block = Block::default().style(Style::default().bg(Color::Red))
                                         .borders(Borders::RIGHT | Borders::LEFT);
        let info = Paragraph::new(info_text).block(info_block);
        
        f.render_widget(code, code_chunk);
        f.render_widget(checkpoints, checkpoints_chunk);
        f.render_widget(info, footer_chunk);
}