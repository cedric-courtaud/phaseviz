use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::rc::Rc;

pub struct CodeLoc {
    pub file_name: Option<Rc<String>>,
    pub function_name: Option<Rc<String>>,
    pub line_nb: u32,
    pub addr_range: (u64, u64),
    pub checkpoints: Vec<u32>
}


impl CodeLoc {
    pub fn new(file_name: Option<Rc<String>>, function_name: Option<Rc<String>>, line_nb: u32, start_addr: u64, end_addr: u64, checkpoints: Vec<u32>) -> CodeLoc {
        CodeLoc {
            file_name: file_name,
            function_name: function_name, 
            line_nb: line_nb,
            addr_range: (start_addr, end_addr),
            checkpoints: checkpoints
        }
    }
}

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

impl Ord for CodeLoc {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp_file =cmp_option_helper(&self.file_name, &other.file_name);
        
        if cmp_file  == Ordering::Equal {
            let cmp_function = cmp_option_helper(&self.function_name, &other.function_name);
            
            if cmp_function == Ordering::Equal {
                return self.line_nb.cmp(&other.line_nb);
            }

            return cmp_function;
        }

        return cmp_file
    }
}

impl PartialOrd for CodeLoc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CodeLoc {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name 
            && self.function_name == other.function_name
            && self.line_nb == other.line_nb
            && self.addr_range == other.addr_range
            && self.checkpoints == other.checkpoints
    }
}

impl Eq for CodeLoc {}

impl Debug for CodeLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodeLoc")
         .field("file_name", &self.file_name)
         .field("function_name", &self.function_name)
         .field("line_nb", &self.line_nb)
         .field("addr_range", &self.addr_range)
         .field("checkpoints",&self.checkpoints)
         .finish()
    }
}

pub struct Profile {
    pub checkpoints: Vec<String>, 
    pub code_loc: Vec<CodeLoc>,
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.checkpoints.eq(&other.checkpoints) && self.code_loc.eq(&other.code_loc)
    }
}

impl Eq for Profile {}

impl Debug for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Profile")
         .field("checkpoints", &self.checkpoints)
         .field("code_loc", &self.code_loc)
         .finish()
    }
}


#[cfg(test)]
mod tests {
    use crate::profile::CodeLoc;
    use std::rc::Rc;

    #[test]
    fn code_loc_eq(){
        let mut c1 = CodeLoc::new(Some(Rc::new(String::from("a"))), Some(Rc::new(String::from("f"))), 1, 0, 0, vec!());
        let mut c2 = CodeLoc::new(Some(Rc::new(String::from("a"))), Some(Rc::new(String::from("f"))), 1, 0, 0, vec!());
        let c3 = CodeLoc::new(Some(Rc::new(String::from("a"))), Some(Rc::new(String::from("f"))), 1, 0, 1, vec!());
        let c4 = CodeLoc::new(Some(Rc::new(String::from("a"))), Some(Rc::new(String::from("f"))), 1, 0, 1, vec!(1));

        assert_eq!(c1, c2);
        assert_ne!(c3, c4);
        assert_ne!(c2, c3);
        
        c1 = CodeLoc::new(None, Some(Rc::new(String::from("f"))), 1, 0, 0, vec!());
        assert_ne!(c1, c2);

        c2 = CodeLoc::new(None, None, 1, 0, 0, vec!());
        assert_ne!(c1, c2);
    }

    #[test]
    fn code_loc_cmp() {
        let c1 = CodeLoc::new(Some(Rc::new(String::from("a"))), Some(Rc::new(String::from("f1"))), 1, 0, 0, vec!());
        let c2 = CodeLoc::new(Some(Rc::new(String::from("a"))), Some(Rc::new(String::from("f1"))), 2, 0, 0, vec!());
        let c3 = CodeLoc::new(Some(Rc::new(String::from("a"))), Some(Rc::new(String::from("f2"))), 1, 0, 0, vec!());
        let c4 = CodeLoc::new(Some(Rc::new(String::from("b"))), Some(Rc::new(String::from("f2"))), 1, 0, 0, vec!());
        let c5 = CodeLoc::new(None, Some(Rc::new(String::from("f2"))), 1, 0, 0, vec!());
        let c6 = CodeLoc::new(None, None, 1, 0, 0, vec!());

        assert_eq!(c1, c1);
        assert!(c1 < c2);
        assert!(c2 < c3);
        assert!(c3 < c4);
        assert!(c4 < c5);
        assert!(c5 < c6);
    }

}