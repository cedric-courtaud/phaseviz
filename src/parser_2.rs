use crate::p2::{Profile, LineInfo, FileInfo, PathInfo, ProfileItem};
use pest::{Parser};

use std::path::Path;
use std::fs::{read_to_string};
use std::u64;
use std::rc::Rc;
use std::collections::BTreeSet;
use std::cell::RefCell;

#[derive(Parser)]
#[grammar = "profile.pest"]
struct ProfileParser;

impl Profile {
    fn parse_checkpoint_name(p: pest::iterators::Pair<Rule>) -> Option<String> {
        for field in p.into_inner() {
            match field.as_rule() {
                Rule::checkpoint_name => {
                    let s = String::from(field.as_str());
                    return Some(s)
                }
                _ => {}
            }
        }

        None
    }

    fn parse_checkpoint_id_section(section: pest::iterators::Pair<Rule>, checkpoints: &mut Vec<String>) {
        for line in section.into_inner() {
            match line.as_rule() {
                Rule::checkpoint_line => {
                    checkpoints.push(Profile::parse_checkpoint_name(line).unwrap());
                }
                _ => {}
            }
        }
    }

    fn parse_code_loc_line(line: pest::iterators::Pair<Rule>, file_section: Rc<RefCell<FileInfo>>, function_name: &Rc<String>, items: &RefCell<BTreeSet<ProfileItem>>) {
        let mut min_addr:u64 = 0;
        let mut max_addr:u64 = 0;
        let mut line_nb:usize = 0;
        let mut checkpoints: Option<BTreeSet<u32>> = None;

        for field in line.into_inner() {
            match field.as_rule() {
                Rule::line_nb => {
                    line_nb = field.into_inner()
                                .as_str()
                                .trim()
                                .parse()
                                .unwrap();
                }

                Rule::addr_range => {
                    let pair:Vec<&str> = field.into_inner().map(|x| x.as_str()).collect();

                    let mut without_prefix = pair[0].trim_start_matches("0x");
                    min_addr = u64::from_str_radix(without_prefix, 16).unwrap();
                    
                    without_prefix = pair[1].trim_start_matches("0x");
                    max_addr = u64::from_str_radix(without_prefix, 16).unwrap();
                }

                Rule::checkpoint_list => {
                    checkpoints = Some(field.into_inner()
                                        .as_str()
                                        .split(" ")
                                        .into_iter()
                                        .map(|token| token.trim().parse::<u32>().unwrap())
                                        .collect());
                }

                _ => {}
            }
        }

        let l = LineInfo::new(line_nb, (min_addr, max_addr), None, Some(function_name.clone()), file_section.borrow().has_debug_info, checkpoints.unwrap());

        items.borrow_mut().insert(ProfileItem::Line(file_section, l));
    }

    fn parse_function_section(section: pest::iterators::Pair<Rule>, file_section: Rc<RefCell<FileInfo>>, items: &RefCell<BTreeSet<ProfileItem>>) {
        let mut function_name = Rc::new(String::from("???"));

        for line in section.into_inner() {
            match line.as_rule() {
                Rule::function_line => {
                    function_name = Rc::new(String::from(line.into_inner().as_str()));
                }

                Rule::code_loc_line => {
                    Profile::parse_code_loc_line(line, file_section.clone(), &function_name, items);
                }

                _ => {}
            }
        }
    }

    fn parse_file_section(section: pest::iterators::Pair<Rule>, items: &RefCell<BTreeSet<ProfileItem>>) {
        let mut pairs = section.into_inner();

        let line = pairs.next().unwrap();

        let fl;

        let item = match line.as_rule() {
                Rule::file_line => {
                    
                    let filename = String::from(line.into_inner().as_str());
                    let path = PathInfo::new("".to_string(), filename);
                    fl = Some(Rc::new(RefCell::new(FileInfo::new(path))));
                    items.borrow_mut().insert(ProfileItem::File(fl.as_ref().unwrap().clone()));
                }

                _ => {unreachable!();}
        };

        for line in pairs {
            match line.as_rule() {
                Rule::function_section => {

                    Profile::parse_function_section(line, fl.as_ref().unwrap().clone(), items);
                }
                _ => {unreachable!()}
            }
        }
    }

    fn parse_code_locs_section(section: pest::iterators::Pair<Rule>, items: &RefCell<BTreeSet<ProfileItem>>) {
        for line in section.into_inner() {
            match line.as_rule() {
                Rule::file_section => {
                   Profile::parse_file_section(line, items);
                }
                _ => {unreachable!()}
            }
        }
    }

    pub fn parse<P: AsRef<Path>>(path: P) -> Self {
        let mut checkpoints = Vec::<String>::new();
        let items = RefCell::new(BTreeSet::new());

        let unparsed_file = read_to_string(path).unwrap();

        let ast = ProfileParser::parse(Rule::file, &unparsed_file).expect("Parse error")
                                                                      .next()
                                                                      .unwrap();
                                
        for section in ast.into_inner() {

            match section.as_rule() {
                Rule::checkpoints_section => { Profile::parse_checkpoint_id_section(section, &mut checkpoints) }

                Rule::codelocs_section => { Profile::parse_code_locs_section(section, &items)}
                
                _ => {}
            }
        }

        Profile {
            items: items.into_inner()
        }
    }
}

#[cfg(test)]
mod tests {
    // use crate::profile::{Profile, FileSection, CodeLine};
    use crate::p2::{Profile, LineInfo, FileInfo, PathInfo, ProfileItem};

    use std::rc::Rc;
    use std::collections::BTreeSet;
    use std::cell::RefCell;

    pub fn asset_memviz_checkpoint_28516() -> Profile {
        let mut items = BTreeSet::new();

        let file = String::from("assets/test/hello/hello.c");
        let func = Rc::new(String::from("main"));

        let f  = Rc::new(RefCell::new(FileInfo::new(PathInfo::new("".to_string(), file))));
        let fi = ProfileItem::File(f.clone());

        items.insert(fi);
        items.insert(ProfileItem::Line(f.clone(), LineInfo::new(9,  (0x1089ac, 0x1089c4), None, Some(func.clone()), true, bt_set!(0))));
        items.insert(ProfileItem::Line(f.clone(), LineInfo::new(11, (0x1089c6, 0x1089cb), None, Some(func.clone()), true, bt_set!(0))));
        items.insert(ProfileItem::Line(f.clone(), LineInfo::new(13, (0x1089d1, 0x108a29), None, Some(func.clone()), true, bt_set!(0, 1))));
        items.insert(ProfileItem::Line(f.clone(), LineInfo::new(15, (0x108a2d, 0x108a34), None, Some(func.clone()), true, bt_set!(1))));
        items.insert(ProfileItem::Line(f.clone(), LineInfo::new(19, (0x108a4e, 0x108a55), None, Some(func.clone()), true, bt_set!(1))));
        
        Profile {
            items: items
        }
    }

    #[test]
    fn parse_memviz_checkpoint_28516(){
        let profile = Profile::parse("assets/test/memviz.chekpoint.28516");
        
        assert_eq!(profile.items, asset_memviz_checkpoint_28516().items);
    }
}