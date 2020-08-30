use crate::model::profile::{ProfileItem};

use tui::{
    widgets::{Paragraph},
    text::{Text, Span, Spans},
    backend::Backend,
    Frame,
    style::{Style, Color, Modifier}
};

use super::{Panel, help_widget, PanelBox};

fn line_nb_col<'a>(n: usize) -> Span<'a> {
    Span::styled(format!("{:5} ", n), Style::default().bg(Color::DarkGray).add_modifier(Modifier::ITALIC))
}

fn render_code_line <'a>(item: &'a ProfileItem) -> Spans<'a> {
    match item {
        ProfileItem::File(f) => {
            Spans::from(vec!(
                Span::styled(" [fl] ", Style::default().bg(Color::Green)),
                Span::styled(format!("  {:?}", f.borrow().path.expand()), Style::default().add_modifier(Modifier::BOLD).add_modifier(Modifier::ITALIC))
            ))
        } 
        ProfileItem::Line(f,l) => {
            if l.line_content.is_some() {
                Spans::from(vec!(
                    line_nb_col(l.nb),
                    Span::raw(format!("  {}", l.line_content.as_ref().unwrap())),
                ))
            }
            else {
            Spans::from(vec![
                line_nb_col(l.nb),
                Span::styled(format!("  in function: "), Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
                Span::styled(format!("{}", l.function.as_ref().unwrap()), Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC).add_modifier(Modifier::BOLD))
            ])
            }
        }
    }
}

pub struct SourcePanel<'a> {
    help: Vec<(&'a str, &'a str)>,
}

pub struct SourceContext<'a> {
    pub pbox: PanelBox<'a>
}

impl<'a> SourcePanel<'a> {
    pub fn new(help: Vec<(&'a str, &'a str)>) -> SourcePanel<'a> {
        SourcePanel{
            help: help,
        }
    }
}

impl<'a> Panel<'a> for SourcePanel<'a> {
    type Context = SourceContext<'a>;

    fn get_context<I>(&'a self, _items: I, p: PanelBox<'a>) -> Self::Context where I: AsRef<[&'a ProfileItem]>{
        SourceContext {
            pbox: p
        }
    }
    
    fn render_header<B, I>(&'a self, f: &mut Frame<B>, items: I, ctx: &Self::Context) where B: Backend, I: AsRef<[&'a ProfileItem]> {
        let txt = if items.as_ref().is_empty() {
            Text::from("")
        } else {
            if items.as_ref()[0].is_file() {
                Text::from("")
            } else {
                let spans = Spans::from(vec!(
                    Span::raw(""),
                    Span::styled(format!("In file {:?}", items.as_ref()[0].get_file_info().borrow().path.expand()), Style::default())
                ));

                Text::from(spans)
            }
        };

        let p = Paragraph::new(txt).block(ctx.pbox.header.block.clone());


        f.render_widget(p, ctx.pbox.header.rect);
    }

    fn render_body<B, I>(&'a self, f: &mut Frame<B>, items: I, ctx: &Self::Context) where B: Backend, I: AsRef<[&'a ProfileItem]>{
        let mut checkpoint_lines:Vec<Spans> = vec!();

        for item in items.as_ref().clone() {
            checkpoint_lines.push(render_code_line(item));
        }
        
        let p = Paragraph::new(Text::from(checkpoint_lines)).block(ctx.pbox.body.block.clone());
        f.render_widget(p, ctx.pbox.body.rect);
    }


    fn render_help<B, I>(&'a self, f: &mut Frame<B>, _items: I, ctx: &Self::Context) where B: Backend, I: AsRef<[&'a ProfileItem]> {
        let w = help_widget(&self.help).block(ctx.pbox.footer.block.clone());
        
        f.render_widget(w, ctx.pbox.footer.rect);
    }
}