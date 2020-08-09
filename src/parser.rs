use crate::profile::{Profile, CodeLoc};
use pest::{Parser};

use std::path::Path;
use std::fs::{read_to_string};
use std::u64;
use std::rc::Rc;

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

    fn check_name(name: &Rc<String>) -> Option<Rc<String>> {
        if **name == "???" {
            return None;
        }

        Some(Rc::clone(name))
    }

    fn parse_code_loc_line<'a>(line: pest::iterators::Pair<Rule>, code_locs: &mut Vec<CodeLoc>, filename: &Rc<String>, function_name: &Rc<String>) {
        let mut min_addr:u64 = 0;
        let mut max_addr:u64 = 0;
        let mut line_nb:u32 = 0;
        let mut checkpoints: Option<Vec<u32>> = None;

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

        let loc = CodeLoc::new(Profile::check_name(filename), 
                               Profile::check_name(function_name), 
                               line_nb.clone(), 
                               min_addr, 
                               max_addr, 
                               checkpoints.unwrap());

        code_locs.push(loc);

    }

    fn parse_function_section(section: pest::iterators::Pair<Rule>, code_locs: &mut Vec<CodeLoc>, filename: &Rc<String>) {
        let mut function_name = Rc::new(String::from("???"));

        for line in section.into_inner() {
            match line.as_rule() {
                Rule::function_line => {
                    function_name = Rc::new(String::from(line.into_inner().as_str()));
                }

                Rule::code_loc_line => {
                    Profile::parse_code_loc_line(line, code_locs, filename, &function_name);
                }
                _ => {}
            }
        }
    }

    fn parse_file_section(section: pest::iterators::Pair<Rule>, code_locs: &mut Vec<CodeLoc>) {
        let mut filename = Rc::new(String::from("???"));

        for line in section.into_inner() {
            match line.as_rule() {
                Rule::file_line => {
                    filename = Rc::new(String::from(line.into_inner().as_str()));
                }

                Rule::function_section => {
                    Profile::parse_function_section(line, code_locs, &filename);
                }
                _ => {}
            }
        }

    }

    fn parse_code_locs_section(section: pest::iterators::Pair<Rule>, code_locs: &mut Vec<CodeLoc>) {
        for line in section.into_inner() {
            match line.as_rule() {
                Rule::file_section => {
                    Profile::parse_file_section(line, code_locs);
                }
                _ => {println!("----- {:?}", line)}
            }
        }
    }

    pub fn parse<P: AsRef<Path>>(path: P) -> Self {
        let mut checkpoints = Vec::<String>::new();
        let mut code_locs = Vec::<CodeLoc>::new();

        let unparsed_file = read_to_string(path).unwrap();

        let ast = ProfileParser::parse(Rule::file, &unparsed_file).expect("Parse error")
                                                                      .next()
                                                                      .unwrap();
                                
        for section in ast.into_inner() {

            match section.as_rule() {
                Rule::checkpoints_section => { Profile::parse_checkpoint_id_section(section, &mut checkpoints) }
                Rule::codelocs_section => { Profile::parse_code_locs_section(section, &mut code_locs)}

                _ => {}
            }
        }

        code_locs.sort();

        Profile {
            checkpoints: checkpoints,
            code_loc: code_locs
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::{CodeLoc, Profile};
    use std::rc::Rc;

    pub fn asset_memviz_checkpoint_28516() -> Profile {
        let file = Rc::new(String::from("hello/hello.c"));
        let func = Rc::new(String::from("main"));

        Profile {
            checkpoints: vec!(
                "memviz_begin",
                "Before_hello"
            ).into_iter()
             .map(|s| String::from(s))
             .collect(),

            code_loc: vec!(
                CodeLoc::new(Some(file.clone()), Some(func.clone()),  9, 0x1089ac, 0x1089c4, vec!(0)),
                CodeLoc::new(Some(file.clone()), Some(func.clone()), 11, 0x1089c6, 0x1089cb, vec!(0)),
                CodeLoc::new(Some(file.clone()), Some(func.clone()), 13, 0x1089d1, 0x108a29, vec!(0, 1)),
                CodeLoc::new(Some(file.clone()), Some(func.clone()), 15, 0x108a2d, 0x108a34, vec!(1)),
                CodeLoc::new(Some(file.clone()), Some(func.clone()), 19, 0x108a4e, 0x108a55, vec!(1))
            )
        }
    }

    #[test]
    fn parse_memviz_checkpoint_28516(){
        let profile = Profile::parse("assets/test/memviz.chekpoint.28516");
        
        assert_eq!(profile, asset_memviz_checkpoint_28516());
    }

}