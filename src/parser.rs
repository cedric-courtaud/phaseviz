use crate::profile::{Profile, CodeLine, FileSection};
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

    fn parse_code_loc_line<'a>(line: pest::iterators::Pair<Rule>, file_section: &mut FileSection, function_name: &Rc<String>) {
        let mut min_addr:u64 = 0;
        let mut max_addr:u64 = 0;
        let mut line_nb:usize = 0;
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

        file_section.lines.insert(CodeLine::new(line_nb, (min_addr, max_addr), None, Some(function_name.clone()), file_section.has_debug_info, checkpoints.unwrap()));
    }

    fn parse_function_section(section: pest::iterators::Pair<Rule>, file_section: &mut FileSection) {
        let mut function_name = Rc::new(String::from("???"));

        for line in section.into_inner() {
            match line.as_rule() {
                Rule::function_line => {
                    function_name = Rc::new(String::from(line.into_inner().as_str()));
                }

                Rule::code_loc_line => {
                    Profile::parse_code_loc_line(line, file_section, &function_name);
                }

                _ => {}
            }
        }
    }

    fn parse_file_section(section: pest::iterators::Pair<Rule>) -> FileSection {
        let mut ret: Option<FileSection> = None;

        let mut pairs = section.into_inner();

        let line = pairs.next().unwrap();

        match line.as_rule() {
                Rule::file_line => {
                    let filename = String::from(line.into_inner().as_str());
                    let has_debug_info = filename == "???";
                    ret = Some(FileSection::new(filename, has_debug_info));
                }

                _ => {}
        }

        let mut s = ret.unwrap();

        for line in pairs {
            match line.as_rule() {
                Rule::function_section => {
                    Profile::parse_function_section(line, &mut s);
                }

                _ => {}
            }
        }

        s
    }

    fn parse_code_locs_section(section: pest::iterators::Pair<Rule>, file_sections: &mut Vec<FileSection>) {
        for line in section.into_inner() {
            match line.as_rule() {
                Rule::file_section => {
                    file_sections.push(Profile::parse_file_section(line));
                }
                _ => {println!("----- {:?}", line)}
            }
        }
    }

    pub fn parse<P: AsRef<Path>>(path: P) -> Self {
        let mut checkpoints = Vec::<String>::new();
        let mut file_sections = Vec::<FileSection>::new();

        let unparsed_file = read_to_string(path).unwrap();

        let ast = ProfileParser::parse(Rule::file, &unparsed_file).expect("Parse error")
                                                                      .next()
                                                                      .unwrap();
                                
        for section in ast.into_inner() {

            match section.as_rule() {
                Rule::checkpoints_section => { Profile::parse_checkpoint_id_section(section, &mut checkpoints) }
                Rule::codelocs_section => { Profile::parse_code_locs_section(section, &mut file_sections)}

                _ => {}
            }
        }

        file_sections.sort();

        Profile {
            checkpoints: checkpoints,
            file_sections: file_sections
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::{Profile, FileSection, CodeLine};
    use std::rc::Rc;

    pub fn asset_memviz_checkpoint_28516() -> Profile {
        let file = String::from("assets/test/hello/hello.c");
        let func = Rc::new(String::from("main"));

        let mut files = FileSection::new(file, true);
        
        files.lines.insert(CodeLine::new(9,  (0x1089ac, 0x1089c4), None, Some(func.clone()), true, vec!(0)));
        files.lines.insert(CodeLine::new(11, (0x1089c6, 0x1089cb), None, Some(func.clone()), true, vec!(0)));
        files.lines.insert(CodeLine::new(13, (0x1089d1, 0x108a29), None, Some(func.clone()), true, vec!(0, 1)));
        files.lines.insert(CodeLine::new(15, (0x108a2d, 0x108a34), None, Some(func.clone()), true, vec!(1)));
        files.lines.insert(CodeLine::new(19, (0x108a4e, 0x108a55), None, Some(func.clone()), true, vec!(1)));
        
        Profile {
            checkpoints: vec!(
                "memviz_begin",
                "Before_hello"
            ).into_iter()
             .map(|s| String::from(s))
             .collect(),

            file_sections: vec!(files)
        }
    }

    #[test]
    fn parse_memviz_checkpoint_28516(){
        let profile = Profile::parse("assets/test/memviz.chekpoint.28516");
        
        assert_eq!(profile, asset_memviz_checkpoint_28516());
    }

}