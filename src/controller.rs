use crate::model::profile::{Profile, ProfileItem};
use std::io::{stdin, stdout};

use termion::{event::Key, input::TermRead, raw::IntoRawMode};

use tui::backend::TermionBackend;
use tui::Terminal;

pub struct App<'a> {
    pub profile: &'a Profile,
    pub items: Vec<&'a ProfileItem>,
    y_pos: usize,
    height: u16,
    should_quit: bool,
}

impl<'a> App<'a> {
    fn scroll_up(&mut self, n: u16) {
        if self.y_pos >= (n as usize) {
            self.y_pos -= n as usize;
        }
    }

    fn scroll_down(&mut self, n: u16) {
        if (self.get_item_count() > 0)
            && (self.get_item_count() > self.height as usize)
            && (self.y_pos < (self.get_item_count() - (n + self.height - 1) as usize))
        {
            self.y_pos += n as usize;
        }
    }

    fn quit(&mut self) {
        self.should_quit = true
    }

    fn wait_for_command(&mut self) {
        match stdin().lock().keys().next().unwrap().unwrap() {
            Key::Up => self.scroll_up(1),
            Key::Down => self.scroll_down(1),
            Key::Char('q') => self.quit(),
            _ => {}
        }
    }

    pub fn get_y_pos(&self) -> usize {
        if self.get_item_count() <= self.height as usize {
            return 0;
        }

        if self.y_pos >= (self.get_item_count() - (self.height as usize)) {
            return self.get_item_count() - (self.height as usize);
        }
        return self.y_pos;
    }

    pub fn get_item_count(&self) -> usize {
        self.items.len()
    }

    pub fn set_height(&mut self, height: u16) {
        self.height = height
    }

    pub fn new(profile: &Profile) -> App {
        App {
            profile: profile,
            should_quit: false,
            items: profile.items.iter().collect(),
            y_pos: 0,
            height: 0,
        }
    }

    pub fn run(&mut self) {
        let stdout = stdout();
        let stdout = stdout.lock().into_raw_mode().unwrap();
        let backend = TermionBackend::new(stdout);

        self.should_quit = false;

        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();

        while !self.should_quit {
            let _ = terminal.draw(|f| crate::ui::draw(f, self));
            self.wait_for_command();
        }
    }
}

#[cfg(test)]
mod tests {
    /*
    use crate::profile::{Profile, FileInfo, ProfileItem, LineInfo};
    use crate::app::{ProfileItem};

    #[test]
    fn test_iter_items(){
        let profile = Profile::parse("assets/test/memviz.chekpoint.28516");

        let items:Vec<ProfileItem> = profile.iter_items().collect();
        let f = &profile.file_sections.first().unwrap();
        let lines: Vec<&CodeLine> = f.lines.iter().collect();

        let expected = vec![
            ProfileItem::FileHeader(&profile.file_sections.first().unwrap()),
            ProfileItem::FunctionLine(lines[0]),
            ProfileItem::FunctionLine(lines[1]),
            ProfileItem::FunctionLine(lines[2]),
            ProfileItem::FunctionLine(lines[3]),
            ProfileItem::FunctionLine(lines[4]),
        ];

        for (i, item) in items.iter().enumerate() {
            assert_eq!(&expected[i], item)
        }
    }*/
}
