use crate::profile::{Profile, FileSection, CodeLine};
use std::io::{stdout, stdin};
use std::fmt;

use termion::{
    event::Key,
    input::TermRead,
    raw::IntoRawMode
};

use tui::backend::TermionBackend;
use tui::Terminal;

#[allow(dead_code)]
pub enum ProfileItem<'a> {
    FileHeader(&'a FileSection),
    CodeLine(&'a CodeLine),
    FunctionLine(&'a CodeLine)
}

impl<'a> fmt::Debug for ProfileItem<'a>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileItem::FileHeader(h) => {
                return f.debug_struct("FileHeader")
                        .field("inner", h)
                        .finish()
            },
            ProfileItem::FunctionLine(l) => {
                return f.debug_struct("FunctionLine")
                        .field("inner", l)
                        .finish()
            },
            ProfileItem::CodeLine(l) => {
                return f.debug_struct("CodeLine")
                        .field("inner", l)
                        .finish()
            }
        }
    }
}

impl<'a> PartialEq for ProfileItem<'a> {
    fn eq(&self, other: &ProfileItem<'_>) -> bool{
        match (self, other) {
            (ProfileItem::FileHeader(f1), ProfileItem::FileHeader(f2)) => {
                return f1 == f2
            }
            (ProfileItem::FunctionLine(f1), ProfileItem::FunctionLine(f2)) => {
                return f1 == f2
            }
            (ProfileItem::CodeLine(l1), ProfileItem::CodeLine(l2)) => {
                return l1 == l2
            }
            (_,_) => return false
        }
    }
}


pub struct ProfileItemIterator<'a> {
    line_iterator: Option<std::collections::btree_set::Iter<'a, CodeLine>>,
    file_iterator: std::slice::Iter<'a, FileSection>,
    curr_file: Option<&'a FileSection>,
}

impl<'a> ProfileItemIterator<'a> {
    fn move_to_next_file(&mut self) -> Option<ProfileItem<'a>> {
        self.curr_file = self.file_iterator.next();

        match self.curr_file {
            Some(f) => {
                self.line_iterator = Some(f.lines.iter());
                return Some(ProfileItem::FileHeader(f))
            }

            None => return None
        }
    }

    fn new(profile: &'a Profile) -> Self {
        ProfileItemIterator {
            file_iterator: profile.file_sections.iter(),
            curr_file: None,
            line_iterator: None,
        }
    }
}

impl<'a> Iterator for ProfileItemIterator<'a> {
    type Item = ProfileItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_file == None {
            return self.move_to_next_file();
        }

        match self.line_iterator.as_mut().unwrap().next() {
            Some(line) => {
                match line.line_content.as_ref() {
                    Some(_) => return Some(ProfileItem::CodeLine(line)),
                    None => return Some(ProfileItem::FunctionLine(line)),
                }
            }

            None => return self.move_to_next_file()
        }
    }
}

impl<'a> Profile {
    pub fn iter_items(&'a self) -> ProfileItemIterator<'a> {
        ProfileItemIterator::new(self)
    }
}

pub struct App<'a> {
    pub profile: &'a Profile,
    pub items: Vec<ProfileItem<'a>>,
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
        if (self.get_item_count() > 0) && 
           (self.get_item_count() > self.height as usize) &&
           (self.y_pos < (self.get_item_count() - (n + self.height - 1) as usize)) {
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
            return self.get_item_count() - (self.height as usize)
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
        App{
            profile: profile,
            should_quit: false,
            items: profile.iter_items().collect(),
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
    use crate::profile::{Profile, CodeLine};
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

    }
}
