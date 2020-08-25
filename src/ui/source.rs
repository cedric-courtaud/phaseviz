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

fn line_nb_col<'a>(n: usize) -> Span<'a> {
    Span::styled(format!("{:5} ", n), Style::default().bg(Color::DarkGray).add_modifier(Modifier::ITALIC))
}

fn render_code_line <'a>(item: &'a ProfileItem) -> Spans<'a> {
    match item {
        ProfileItem::FileHeader(f) => {
            return Spans::from(vec![
                Span::styled(" [fl] ", Style::default().bg(Color::Green)),
                Span::styled(format!("  {}", f.name), Style::default().add_modifier(Modifier::BOLD).add_modifier(Modifier::ITALIC))
            ])
        }

        ProfileItem::FunctionLine(l) => {
            return Spans::from(vec![
                line_nb_col(l.nb),
                Span::styled(format!("  in function: "), Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                Span::styled(format!("{}", l.function.as_ref().unwrap()), Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC).add_modifier(Modifier::BOLD))
            ])
        }

        ProfileItem::CodeLine(l) => {
            return Spans::from(vec![
                line_nb_col(l.nb),
                Span::raw(format!("  {}", l.line_content.as_ref().unwrap())),
            ])
        }
    }

}

pub struct SourcePanel<'a> {
    help: Vec<(&'a str, &'a str)>
}

impl<'a> SourcePanel<'a> {
    pub fn new(help: Vec<(&'a str, &'a str)>) -> SourcePanel<'a> {
        SourcePanel{
            help: help
        }
    }
}

impl<'a> Panel<'a> for SourcePanel<'a> {
    fn render_header<B, I>(&'a self,f: &mut Frame<B>, _items: I, rect: Rect, _block: Block) where B: Backend, I: AsRef<[ProfileItem<'a>]>{
    }

    fn render_body<B, I>(&'a self,f: &mut Frame<B>, items: I, rect: Rect, block: Block) where B: Backend, I: AsRef<[ProfileItem<'a>]>{
        let mut checkpoint_lines:Vec<Spans> = vec!();

        for item in items.as_ref().clone() {
            checkpoint_lines.push(render_code_line(item));
        }
        
        let p = Paragraph::new(Text::from(checkpoint_lines)).block(block);
        f.render_widget(p, rect);
    }


    fn render_help<B>(&'a self,f: &mut Frame<B>, rect: Rect, block: Block) where B: Backend {
        let w = help_widget(&self.help).block(block);
        
        f.render_widget(w, rect);
    }
}