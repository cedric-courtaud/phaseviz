use crate::model::profile::ProfileItem;

use tui::{
    backend::Backend,
    text::{Span, Spans, Text},
    widgets::{Borders, Paragraph},
    Frame,
};

use super::{Panel, PanelBox};


pub struct InstAddrPanel<'a> {
       _help: Vec<(&'a str, &'a str)>,
}

pub struct InstAddrContext<'a> {
    pub pbox: PanelBox<'a>,
}

impl<'a> InstAddrPanel<'a> {
    pub fn new(help: Vec<(&'a str, &'a str)>) -> InstAddrPanel<'a> {
        InstAddrPanel { _help: help }
    }
}

impl<'a> Panel<'a> for InstAddrPanel<'a> {
    type Context = InstAddrContext<'a>;

    fn get_context<I>(&'a self, _items: I, p: PanelBox<'a>) -> Self::Context
    where
        I: AsRef<[&'a ProfileItem]>,
    {
        InstAddrContext { pbox: p }
    }

    fn render_header<B, I>(&'a self, f: &mut Frame<B>, _items: I, ctx: &Self::Context)
    where
        B: Backend,
        I: AsRef<[&'a ProfileItem]>,
    {
        let p = Paragraph::new(Span::from(format!("{:^30}", "Inst addr range")))
            .block(ctx.pbox.header.block.clone());

        f.render_widget(p, ctx.pbox.header.rect);
    }

    fn render_body<B, I>(&'a self, f: &mut Frame<B>, items: I, ctx: &Self::Context)
    where
        B: Backend,
        I: AsRef<[&'a ProfileItem]>,
    {
        let mut lines: Vec<Spans> = vec![];

        for item in items.as_ref() {
            match item {
                ProfileItem::File(_) => lines.push(Spans::from(vec![])),
                ProfileItem::Line(_, l) => {
                    if l.addr_range == (0, 0) {
                        lines.push(Spans::from(vec![]));
                    } else {
                        let spans = Spans::from(vec![
                            Span::raw(format!("  {:>010x}", l.addr_range.0)),
                            Span::raw(format!(" -> ")),
                            Span::raw(format!("{:<010x} ", l.addr_range.1)),
                        ]);
                        lines.push(spans);
                    }
                }
            }
        }

        let p = Paragraph::new(Text::from(lines))
            .block(ctx.pbox.body.block.clone().borders(Borders::NONE));
        f.render_widget(p, ctx.pbox.body.rect);
    }

    fn render_help<B, I>(&'a self, f: &mut Frame<B>, _items: I, ctx: &Self::Context)
    where
        B: Backend,
        I: AsRef<[&'a ProfileItem]>,
    {
        f.render_widget(ctx.pbox.footer.block.clone(), ctx.pbox.footer.rect);
    }
}
