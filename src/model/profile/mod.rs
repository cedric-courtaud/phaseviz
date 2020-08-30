use std::{
    collections::BTreeSet,
    rc::Rc,
    cmp::Ordering,
    cell::{RefCell},
    path::Path,
    marker::PhantomData,
    iter::{Cloned, Peekable}
};

mod parser;

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

/// Path to a source code file
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PathInfo {
    pub directory: String,
    pub file: String
}

impl Ord for PathInfo {
    /// Compare two path with lexicographic order
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_dir = self.directory.cmp(&other.directory);

        if cmp_dir == Ordering::Equal {
            return self.file.cmp(&other.file)
        }

        cmp_dir
    }
}

impl PartialOrd for PathInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PathInfo {
    /// Create a new path info.
    /// Ensures that files whose name is "???" has an empty directory.
    pub fn new(directory: String, file: String) -> PathInfo {
        PathInfo{
            directory: if file == "???" {String::from("")} else {directory},
            file: file,
        }
    }

    pub fn expand(&self) -> std::path::PathBuf {
        Path::new(&self.directory).join(&self.file)
    }

}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FileInfo {
    pub path: PathInfo,
    pub has_debug_info: bool,
    pub checkpoints: BTreeSet<u32>,
}

impl Ord for FileInfo {
    /// Compare two file descriptor
    /// If one of the two operands is an undefined file (a file whose name is "???"), 
    /// then this operand is the smallest file descriptor. Otherwise, we apply lexicographic 
    /// order on the file path
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_name = self.path.cmp(&other.path);

        // We assume that undefined files belongs to the same directory
        if cmp_name != Ordering::Equal {
            if self.path.file == "???" {
                return Ordering::Less
            }
            if other.path.file == "???" {
                return Ordering::Greater
            }
        }
        cmp_name
    }
}

impl PartialOrd for FileInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl FileInfo {
    pub fn new(path: PathInfo) -> FileInfo {
        FileInfo{
            has_debug_info: path.file != "???",
            path: path,
            checkpoints: BTreeSet::new(),
        }
    }

    pub fn get_file_content(&self) -> std::io::Result<String> {
        std::fs::read_to_string(self.path.expand())
    }

}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LineInfo {
    pub nb : usize,
    pub addr_range: (u64, u64),
    pub line_content: Option<String>,
    pub function: Option<Rc<String>>,
    pub checkpoints: BTreeSet<u32>,
    pub has_debug_info: bool
}

impl Ord for LineInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_func = cmp_option_helper(&self.function, &other.function);

        if self.has_debug_info || (cmp_func == Ordering::Equal) {
            return self.nb.cmp(&other.nb)
        }

        cmp_func
    }
}

impl PartialOrd for LineInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl LineInfo {
    pub fn new(nb: usize, addr_range: (u64, u64), line_content: Option<String>, function: Option<Rc<String>>, is_file_available: bool, checkpoints: BTreeSet<u32>) -> LineInfo {
        LineInfo {
            nb: nb,
            addr_range: addr_range,
            line_content: line_content,
            function: function,
            checkpoints: checkpoints,
            has_debug_info: is_file_available,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialOrd)]
pub enum ProfileItem {
    File(Rc<RefCell<FileInfo>>),
    Line(Rc<RefCell<FileInfo>>, LineInfo)
}

impl PartialEq for ProfileItem {
    fn eq(&self, other: &ProfileItem) -> bool{
        match (self, other) {
            (ProfileItem::File(f1), ProfileItem::File(f2)) => {
                return f1 == f2
            }
            (ProfileItem::Line(f1, l1), ProfileItem::Line(f2, l2)) => {
                return (f1 == f2) && (l1 == l2)
            }
            (_,_) => return false
        }
    }
}


impl <'a> Ord for ProfileItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ProfileItem::File(f1), ProfileItem::File(f2)) => f1.cmp(f2),
            (ProfileItem::Line(f1, l1), ProfileItem::Line(f2, l2)) => { 
                if f1 == f2 {
                    l1.cmp(l2)
                } else {
                    f1.cmp(f2)
                }
            }

            (ProfileItem::File(f1), ProfileItem::Line(f2, _)) => {
                if f1 == f2 {
                    Ordering::Less
                } else {
                    f1.cmp(f2)
                }
            }

            (ProfileItem::Line(f1, _), ProfileItem::File(f2)) => {
                if f1 == f2 {
                    Ordering::Greater
                } else {
                    f1.cmp(f2)
                }
            }
        }

    }
}

impl<'a> ProfileItem {
    pub fn is_file(&'a self) -> bool {
        match self {
            ProfileItem::File(_) => true,
            _ => false 
        }
    }

    pub fn is_in_same_file<'b: 'a>(&'a self, other: &'b ProfileItem) -> bool {
        match (self, other) {
            (ProfileItem::Line(f1, _), ProfileItem::File(f2))    => f1 == f2,
            (ProfileItem::File(f2), ProfileItem::Line(f1, l))    => f1 == f2,
            (ProfileItem::File(f1), ProfileItem::File(f2))       => f1 == f2,
            (ProfileItem::Line(f1, _), ProfileItem::Line(f2, _)) => f1 == f2,
        }
    }

    pub fn get_file_info(&'a self) -> Rc<RefCell<FileInfo>> {
        match self {
            ProfileItem::File(f) => f.clone(),
            ProfileItem::Line(f, _) => f.clone()
        }
    }
}

/// An iterator over a file section.
/// A file section comprises all the profile items concerning a source file.
/// The first item is always a ProfileItem::File.
/// It is followed by all the available lines ordered by line number.
/// Right after parsing, lines are not synced with the corresponding source file.  You will
/// have to call `sync_with_fs` to do that.
pub struct FileSection<'a> {
    iter: Box<dyn Iterator<Item=&'a ProfileItem> +'a>,
    phantom: PhantomData<&'a Profile>,
}

impl<'a> FileSection<'a> {
    pub fn new (profile:&'a Profile, f: &'a ProfileItem) -> FileSection<'a> {
        let file  = std::iter::once(f);
        let lines = profile.items.iter().filter(move |item| !item.is_file() && item.is_in_same_file(f));

        FileSection {
            iter: Box::new(file.chain(lines)),
            phantom: PhantomData
        }
    }

    pub fn synced(self) -> SyncedFileSection<'a> {
        SyncedFileSection::new(self)
    }
}

impl<'a> Iterator for FileSection<'a> {
    type Item=&'a ProfileItem;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct SyncedFileSection<'a> {
    file: Rc<RefCell<FileInfo>>,
    section: Peekable<Cloned<FileSection<'a>>>,
    lines: Option<Vec<String>>,
    pos: usize,
}

impl<'a> SyncedFileSection<'a> {
    pub fn new(section: FileSection<'a>) -> SyncedFileSection<'a> {
        let mut new_iter = section.cloned()
                                  .peekable();
        
        let file_info = new_iter.peek()
                                .unwrap()
                                .get_file_info();

        let maybe_content = file_info.borrow()
                                     .get_file_content();

        match maybe_content {
            Ok(content) => {
                SyncedFileSection {
                    file: file_info.clone(),
                    section: new_iter,
                    lines: Some(content.lines().map(String::from).collect()),
                    pos: 0,
                }
            },

            Err(_) => {
                SyncedFileSection {
                    file: file_info.clone(),
                    section: new_iter,
                    lines: None,
                    pos: 0
                }
            }
        }
    }
}

impl <'a> Iterator for SyncedFileSection<'a> {
    type Item = ProfileItem;

    fn next(&mut self) -> Option<Self::Item> {
        // If lines are unavailable this is a copy of the section
        if let None = self.lines {
            return self.section.next()
        }

        // Iteration ends when there is no remaining lines
        if self.pos >= self.lines.as_ref().unwrap().len() {
            return None;
        }

        let curr_item = self.section.peek();

        match curr_item {
            Some(ProfileItem::File(_)) => {
                self.section.next()
            },

            Some(ProfileItem::Line(_, l)) => {
                let nl = if (self.pos + 1) == l.nb {
                    let mut tmp = l.clone();
                    tmp.line_content = Some(self.lines.as_ref().unwrap()[self.pos].clone());
                    self.section.next();
                    tmp
                } else {
                    let mut nl = LineInfo::new(self.pos + 1, (0, 0), None, None, true, bt_set!());
                    nl.line_content = Some(self.lines.as_ref().unwrap()[self.pos].clone());
                    nl
                };
                
    
                self.pos += 1;
                
                Some(ProfileItem::Line(self.file.clone(), nl))
            }

            None => {
                let mut nl = LineInfo::new(self.pos + 1, (0, 0), None, None, true, bt_set!());
                
                nl.line_content = Some(self.lines.as_ref().unwrap()[self.pos].clone());
                
                self.pos += 1;

                Some(ProfileItem::Line(self.file.clone(), nl))
            }
        }
    }
}

/// An iterator over all the profile file sections.
/// File sections are guaranteed to be ordered by FileInfo.
pub struct FileSections<'a> {
    iter: Box<dyn Iterator<Item=FileSection<'a>> + 'a>,
}

impl<'a> FileSections<'a> {
    pub fn new (profile: &'a Profile) -> FileSections<'a> {
        let files = profile.items.iter().filter(|item| item.is_file());
        let iter = files.map(move |file| profile.file_section(file));
        
        FileSections {
            iter: Box::new(iter),
        }
    }
}

impl<'a> Iterator for FileSections<'a> {
    type Item=FileSection<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct SyncedProfileItems <'a> {
    iter: Box<dyn Iterator<Item=ProfileItem> + 'a>,
}

impl<'a> SyncedProfileItems<'a> {
    pub fn new (profile: &'a Profile) -> SyncedProfileItems<'a> {
        let files = profile.items.iter().filter(|item| item.is_file());
        let iter = files.map(move |file| profile.file_section(file));

        SyncedProfileItems {
            iter: Box::new(profile.file_sections()
                                  .map(move |section| section.synced())
                                  .flatten())
        }

    }
}

impl<'a> Iterator for SyncedProfileItems<'a> {
    type Item=ProfileItem;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}



pub struct Profile {
    pub items: BTreeSet<ProfileItem>
}

impl<'a> Profile {
    pub fn file_sections(&'a self) -> FileSections<'a> {
        FileSections::new(self)
    }

    pub fn file_section(&'a self, item: &'a ProfileItem) -> FileSection<'a> {
        FileSection::new(self, item)
    }

    pub fn synced_items(&'a self) -> SyncedProfileItems<'a> {
        SyncedProfileItems::new(self)
    }

    pub fn synced(&self) -> Profile {
        Profile{
            items: self.file_sections()
                       .map(|section| section.synced())
                       .flatten()
                       .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::{LineInfo, PathInfo, FileInfo, ProfileItem, Profile};
    use std::{
        collections::BTreeSet,
        rc::Rc,
        cell::RefCell
    };

    #[test]
    fn line_cmp() {
        let l1 = LineInfo::new(0, (0, 1) , None, Some(Rc::new(String::from("g"))), true, bt_set!());
        let l2 = LineInfo::new(1, (0, 1) , None, Some(Rc::new(String::from("f"))), true, bt_set!());
        let l3 = LineInfo::new(2, (0, 1) , None, Some(Rc::new(String::from("f"))), true, bt_set!());
        let l4 = LineInfo::new(1, (0, 1) , None, Some(Rc::new(String::from("f"))), false, bt_set!());
        let l5 = LineInfo::new(0, (0, 1) , None, Some(Rc::new(String::from("g"))), false, bt_set!());

        assert!(l1 < l2);
        assert!(l2 < l3);
        assert!(l1 < l3);
        assert!(l4 < l5);
    }

    #[test]
    fn path_cmp() {
        let p1 = PathInfo::new("/foo".to_string(), "bar".to_string());
        let p2 = PathInfo::new("/foo".to_string(), "car".to_string());
        let p3 = PathInfo::new("/boo".to_string(), "car".to_string());

        assert!(p1 < p2);
        assert!(p3 < p2);
        assert!(p3 < p1);
    }

    #[test]
    fn path_new(){
        let p1 = PathInfo::new("/foo".to_string(), "bar".to_string());
        let p2 = PathInfo::new("".to_string(), "???".to_string());
        let p3 = PathInfo::new("/foo".to_string(), "???".to_string());

        assert_eq!(p1.directory, "/foo");
        assert_eq!(p1.file, "bar");

        assert_eq!(p2.directory, "");
        assert_eq!(p2.file, "???");
        
        assert_eq!(p3.directory, "");
        assert_eq!(p3.file, "???");

        assert_eq!(p2, p3);
    }

    #[test]
    fn item_cmp(){
        let p1 = PathInfo::new("/foo".to_string(), "bar".to_string());
        let p2 = PathInfo::new("/foo".to_string(), "car".to_string());

        let f1 = FileInfo::new(p1);
        let f2 = FileInfo::new(p2);

        let l1 = LineInfo::new(0, (0, 1) , None, Some(Rc::new(String::from("g"))), true, bt_set!());
        let l2 = LineInfo::new(1, (0, 1) , None, Some(Rc::new(String::from("f"))), true, bt_set!());
        let l3 = LineInfo::new(1, (0, 1) , None, Some(Rc::new(String::from("f"))), true, bt_set!());

        let c1 = Rc::new(RefCell::new(f1));
        let c2 = Rc::new(RefCell::new(f2));

        
        let i1 = ProfileItem::File(c1.clone());
        let i2 = ProfileItem::Line(c1.clone(), l1);
        let i3 = ProfileItem::Line(c1.clone(), l2);
        let i4 = ProfileItem::Line(c2.clone(), l3);

        assert!(i1 < i2);
        assert!(i2 < i3);
        assert!(i3 < i4);
        assert!(i1 < i4);
    }

    #[test]
    fn file_sections() { 
        let p1 = PathInfo::new("/foo".to_string(), "bar".to_string());
        let p2 = PathInfo::new("/foo".to_string(), "car".to_string());

        let f1 = FileInfo::new(p1);
        let f2 = FileInfo::new(p2);

        let l1 = LineInfo::new(0, (0, 1) , None, Some(Rc::new(String::from("g"))), true, bt_set!());
        let l2 = LineInfo::new(1, (0, 1) , None, Some(Rc::new(String::from("f"))), true, bt_set!());
        let l3 = LineInfo::new(1, (0, 1) , None, Some(Rc::new(String::from("f"))), true, bt_set!());

        let c1 = Rc::new(RefCell::new(f1));
        let c2 = Rc::new(RefCell::new(f2));

        let i1 = ProfileItem::File(c1.clone());
        let i2 = ProfileItem::Line(c1.clone(), l1);
        let i3 = ProfileItem::Line(c1.clone(), l2);
        let i4 = ProfileItem::File(c2.clone());
        let i5 = ProfileItem::Line(c2.clone(), l3);

        let mut items = BTreeSet::new();

        items.insert(i1.clone());
        items.insert(i2.clone());
        items.insert(i3.clone());
        items.insert(i4.clone());
        items.insert(i5.clone());

        let profile = Profile{items};

        let mut file_sections = profile.file_sections();

        let mut section = file_sections.next().unwrap();
        assert_eq!(Some(&i1), section.next());
        assert_eq!(Some(&i2), section.next());
        assert_eq!(Some(&i3), section.next());
        assert_eq!(None, section.next());
        
        section = file_sections.next().unwrap();
        assert_eq!(Some(&i4), section.next());
        assert_eq!(Some(&i5), section.next());
        assert_eq!(None, section.next());
    }

    #[test]
    fn sync_with_fs(){
        
        let file = String::from("assets/test/hello/hello.c");
        let func = Rc::new(String::from("main"));
        let path = PathInfo::new("".to_string(), file);

        let f = Rc::new(RefCell::new(FileInfo::new(path)));

        let mut items1 = BTreeSet::new();
        items1.insert(ProfileItem::File(f.clone()));
        items1.insert(ProfileItem::Line(f.clone(), LineInfo::new(9,  (0x1089ac, 0x1089c4), None, Some(func.clone()), true, bt_set!(0))));
        items1.insert(ProfileItem::Line(f.clone(), LineInfo::new(11, (0x1089c6, 0x1089cb), None, Some(func.clone()), true, bt_set!(0))));
        items1.insert(ProfileItem::Line(f.clone(), LineInfo::new(13, (0x1089d1, 0x108a29), None, Some(func.clone()), true, bt_set!(0, 1))));
        items1.insert(ProfileItem::Line(f.clone(), LineInfo::new(15, (0x108a2d, 0x108a34), None, Some(func.clone()), true, bt_set!(1))));
        items1.insert(ProfileItem::Line(f.clone(), LineInfo::new(19, (0x108a4e, 0x108a55), None, Some(func.clone()), true, bt_set!(1))));

        let file_content = f.borrow().get_file_content().unwrap();
        let file_lines: Vec<&str> = file_content.lines().collect();
        
        let mut items2 = BTreeSet::new();
        items2.insert(ProfileItem::File(f.clone()));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(1,  (0, 0), Some(String::from(file_lines[0].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(2,  (0, 0), Some(String::from(file_lines[1].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(3,  (0, 0), Some(String::from(file_lines[2].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(4,  (0, 0), Some(String::from(file_lines[3].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(5,  (0, 0), Some(String::from(file_lines[4].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(6,  (0, 0), Some(String::from(file_lines[5].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(7,  (0, 0), Some(String::from(file_lines[6].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(8,  (0, 0), Some(String::from(file_lines[7].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(9,  (0x1089ac, 0x1089c4), Some(String::from(file_lines[8].clone())), Some(func.clone()), true, bt_set!(0))));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(10, (0, 0), Some(String::from(file_lines[9].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(11, (0x1089c6, 0x1089cb), Some(String::from(file_lines[10].clone())), Some(func.clone()), true, bt_set!(0))));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(12, (0, 0), Some(String::from(file_lines[11].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(13, (0x1089d1, 0x108a29), Some(String::from(file_lines[12].clone())), Some(func.clone()), true, bt_set!(0,1))));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(14, (0, 0), Some(String::from(file_lines[13].clone())), None,true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(15, (0x108a2d, 0x108a34), Some(String::from(file_lines[14].clone())), Some(func.clone()), true, bt_set!(1))));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(16, (0, 0), Some(String::from(file_lines[15].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(17, (0, 0), Some(String::from(file_lines[16].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(18, (0, 0), Some(String::from(file_lines[17].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(19, (0x108a4e, 0x108a55), Some(String::from(file_lines[18].clone())), Some(func.clone()), true, bt_set!(1))));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(20, (0, 0), Some(String::from(file_lines[19].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(21, (0, 0), Some(String::from(file_lines[20].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(22, (0, 0), Some(String::from(file_lines[21].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(23, (0, 0), Some(String::from(file_lines[22].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(24, (0, 0), Some(String::from(file_lines[23].clone())), None, true, bt_set!())));
        items2.insert(ProfileItem::Line(f.clone(), LineInfo::new(25, (0, 0), Some(String::from(file_lines[24].clone())), None, true, bt_set!())));

        let p1 = Profile{
            //checkpoints: vec!(),
            //file_sections: vec!(items1)
            items: items1
        };

        let p2 = Profile{
            items: items2
        };

        for (i1, i2) in p2.items.iter().zip(p1.synced().items.iter()) {
            assert_eq!(i1, i2);
        }

    }
}