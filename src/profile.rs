
use std::collections::BTreeSet;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::rc::Rc;
use std::fs::{read_to_string};


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
    pub has_debug_info: bool
}

impl CodeLine {
    pub fn new(nb: usize, addr_range: (u64, u64), line_content: Option<String>, function: Option<Rc<String>>, is_file_available: bool, checkpoints: Vec<u32>) -> CodeLine {
        CodeLine{
            nb: nb,
            addr_range: addr_range,
            line_content: line_content,
            function: function,
            checkpoints: checkpoints,
            has_debug_info: is_file_available,
        }
    }
}

impl Ord for CodeLine {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_func = cmp_option_helper(&self.function, &other.function);

        if self.has_debug_info || (cmp_func == Ordering::Equal) {
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
         .field("is_file_available", &self.has_debug_info)
         .field("checkpoints",&self.checkpoints)
         .finish()
    }
}

pub struct FileSection {
    pub name: String,
    pub lines: BTreeSet<CodeLine>,
    pub has_debug_info: bool,
}

impl FileSection {
    pub fn new(name: String, has_debug_info: bool) -> FileSection {
        FileSection{
            name: name, 
            lines: BTreeSet::new(),
            has_debug_info: has_debug_info,
        }
    }

    pub fn sync_with_fs(&mut self){
        if !self.has_debug_info {
            return
        }

        let content = read_to_string(&self.name);

        match content {
            Ok(c) => {
                let mut new_set = BTreeSet::<CodeLine>::new();
                
                for (i, line) in c.lines().enumerate() {
                    let first = self.lines.first();

                    match first {
                        Some(codeline) => {
                            if i == (codeline.nb - 1) {
                                let mut tmp = self.lines.pop_first().unwrap();
                                tmp.line_content = Some(String::from(line));
                                new_set.insert(tmp);
                            } else {
                                new_set.insert(CodeLine::new(i + 1, (0, 0), Some(String::from(line)), None, true, vec!()));
                            }
                        }
                        None => {
                                new_set.insert(CodeLine::new(i + 1, (0, 0), Some(String::from(line)), None, true, vec!()));
                        }
                    }
                }

                self.lines = new_set;
            }

            Err(_) => { return }
        }

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

impl Profile {
    pub fn sync_with_fs(&mut self) {
        for section in &mut self.file_sections {
            section.sync_with_fs();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::{Profile, CodeLine, FileSection};
    use std::rc::Rc;

    #[test]
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
    #[test]
    fn profile_expansion() {
        let file = String::from("assets/test/hello/hello.c");
        let func = Rc::new(String::from("main"));

        let mut example = FileSection::new(file.clone(), true);
        
        example.lines.insert(CodeLine::new(9,  (0x1089ac, 0x1089c4), None, Some(func.clone()), true, vec!(0)));
        example.lines.insert(CodeLine::new(11, (0x1089c6, 0x1089cb), None, Some(func.clone()), true, vec!(0)));
        example.lines.insert(CodeLine::new(13, (0x1089d1, 0x108a29), None, Some(func.clone()), true, vec!(0, 1)));
        example.lines.insert(CodeLine::new(15, (0x108a2d, 0x108a34), None, Some(func.clone()), true, vec!(1)));
        example.lines.insert(CodeLine::new(19, (0x108a4e, 0x108a55), None, Some(func.clone()), true, vec!(1)));

        let mut expected = FileSection::new(file.clone(), true);
        let file_content = std::fs::read_to_string(&file).unwrap();
        let file_lines: Vec<&str> = file_content.lines().collect();
        
        expected.lines.insert(CodeLine::new(1,  (0, 0), Some(String::from(file_lines[0].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(2,  (0, 0), Some(String::from(file_lines[1].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(3,  (0, 0), Some(String::from(file_lines[2].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(4,  (0, 0), Some(String::from(file_lines[3].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(5,  (0, 0), Some(String::from(file_lines[4].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(6,  (0, 0), Some(String::from(file_lines[5].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(7,  (0, 0), Some(String::from(file_lines[6].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(8,  (0, 0), Some(String::from(file_lines[7].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(9,  (0x1089ac, 0x1089c4), Some(String::from(file_lines[8].clone())), Some(func.clone()), true, vec!(0)));
        expected.lines.insert(CodeLine::new(10, (0, 0), Some(String::from(file_lines[9].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(11, (0x1089c6, 0x1089cb), Some(String::from(file_lines[10].clone())), Some(func.clone()), true, vec!(0)));
        expected.lines.insert(CodeLine::new(12, (0, 0), Some(String::from(file_lines[11].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(13, (0x1089d1, 0x108a29), Some(String::from(file_lines[12].clone())), Some(func.clone()), true, vec!(0,1)));
        expected.lines.insert(CodeLine::new(14, (0, 0), Some(String::from(file_lines[13].clone())), None,true, vec!()));
        expected.lines.insert(CodeLine::new(15, (0x108a2d, 0x108a34), Some(String::from(file_lines[14].clone())), Some(func.clone()), true, vec!(1)));
        expected.lines.insert(CodeLine::new(16, (0, 0), Some(String::from(file_lines[15].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(17, (0, 0), Some(String::from(file_lines[16].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(18, (0, 0), Some(String::from(file_lines[17].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(19, (0x108a4e, 0x108a55), Some(String::from(file_lines[18].clone())), Some(func.clone()), true, vec!(1)));
        expected.lines.insert(CodeLine::new(20, (0, 0), Some(String::from(file_lines[19].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(21, (0, 0), Some(String::from(file_lines[20].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(22, (0, 0), Some(String::from(file_lines[21].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(23, (0, 0), Some(String::from(file_lines[22].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(24, (0, 0), Some(String::from(file_lines[23].clone())), None, true, vec!()));
        expected.lines.insert(CodeLine::new(25, (0, 0), Some(String::from(file_lines[24].clone())), None, true, vec!()));

        let mut p1 = Profile{
            checkpoints: vec!(),
            file_sections: vec!(example)
        };

        let p2 = Profile{
            checkpoints: vec!(),
            file_sections: vec!(expected)
        };

        p1.sync_with_fs();
        assert_eq!(p1, p2);
    }
}