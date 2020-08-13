use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::rc::Rc;
use std::collections::BTreeSet;

fn cmp_option_helper<T:Ord>(a: &Option<T>, b: &Option<T>) -> Ordering {

    let comp = a.cmp(b);

    if comp != Ordering::Equal {
        if let None = a {
            return Ordering::Greater
        }
        if let None = b {
            return Ordering::Less
        }
    }

    comp
}

pub struct CodeLine {
    pub nb : usize,
    pub addr_range: (u64, u64),
    pub line_content: Option<String>,
    pub function: Option<Rc<String>>,
    pub checkpoints: Vec<u32>,
    pub is_file_available: bool
}

impl CodeLine {
    pub fn new(nb: usize, addr_range: (u64, u64), line_content: Option<String>, function: Option<Rc<String>>, is_file_available: bool, checkpoints: Vec<u32>) -> CodeLine {
        CodeLine{
            nb: nb,
            addr_range: addr_range,
            line_content: line_content,
            function: function,
            checkpoints: checkpoints,
            is_file_available: is_file_available,
        }
    }
}

impl Ord for CodeLine {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_func = cmp_option_helper(&self.function, &other.function);

        if self.is_file_available || (cmp_func == Ordering::Equal) {
            return self.nb.cmp(&other.nb)
        }

        cmp_func
    }
}

impl PartialOrd for CodeLine {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for CodeLine {
    fn eq(&self, other: &Self) -> bool {
        self.nb == other.nb &&
        self.addr_range == other.addr_range &&
        self.function == other.function
    }
}

impl Eq for CodeLine {}

impl Debug for CodeLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodeLine")
         .field("nb", &self.nb)
         .field("addr_range", &self.addr_range)
         .field("function", &self.function)
         .field("is_file_available", &self.is_file_available)
         .field("checkpoints",&self.checkpoints)
         .finish()
    }
}

pub struct FileSection {
    pub name: String,
    pub lines: BTreeSet<CodeLine>,
    pub available: bool,
}

impl FileSection {
    pub fn new(name: String, available: bool) -> FileSection {
        FileSection{
            name: name, 
            lines: BTreeSet::new(),
            available: available,
        }
    }

    pub fn insert_line(&mut self, line: CodeLine) {

    }
}

impl Ord for FileSection {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_name = self.name.cmp(&other.name);

        if cmp_name != Ordering::Equal {
            if self.name == "???" {
                return Ordering::Less
            }
            if other.name == "???" {
                return Ordering::Greater
            }
        }

        cmp_name
    }
}

impl PartialOrd for FileSection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for FileSection {
    fn eq(&self, other: &Self) -> bool {
        (self.name == other.name) && (self.lines == other.lines)
    }
}

impl Eq for FileSection {}

impl Debug for FileSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileSection")
         .field("name", &self.name)
         .field("lines",&self.lines)
         .finish()
    }
}


pub struct Profile {
    pub checkpoints: Vec<String>, 
    pub file_sections: Vec<FileSection>,
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.checkpoints.eq(&other.checkpoints) 
        && self.file_sections.eq(&other.file_sections)
    }
}

impl Eq for Profile {}

impl Debug for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Profile")
         .field("checkpoints", &self.checkpoints)
         .field("code_loc", &self.file_sections)
         .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::{Profile, CodeLine, FileSection};
    use std::rc::Rc;

    fn line_cmp() {
        let l1 = CodeLine::new(0, (0, 1) , None, Some(Rc::new(String::from("g"))), true, vec!());
        let l2 = CodeLine::new(1, (0, 1) , None, Some(Rc::new(String::from("f"))), true, vec!());
        let l3 = CodeLine::new(2, (0, 1) , None, Some(Rc::new(String::from("f"))), true, vec!());
        let l4 = CodeLine::new(1, (0, 1) , None, Some(Rc::new(String::from("f"))), false, vec!());
        let l5 = CodeLine::new(0, (0, 1) , None, Some(Rc::new(String::from("g"))), false, vec!());

        assert!(l1 < l2);
        assert!(l2 < l3);
        assert!(l1 < l3);
        assert!(l4 < l5);
    }
}