use lazy_static::*;
use regex::{Captures, Match, Regex};
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

fn main() {
    println!("Hello, world!");

    // 第一个程序：读文件、正则匹配、文件结构parse、写文件
    let path = r"E:\trunk\Survive\Source\Lua\proto\ds_client\SocialIsland\Pso.proto";
    let file_struct = parse_file(path).unwrap();
    let gen_c2d_file = r"J:\myrust\mytools\src\c2d.lua";
    let gen_d2c_file = r"J:\myrust\mytools\src\d2c.lua";
    write_lua_file(gen_c2d_file, gen_d2c_file, file_struct).unwrap();
}

enum State {
    Normal,
    MessageDefine,
    MessageScopeBegin,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum MessageType {
    Req,
    Rsp,
    Notify,
}

#[derive(Debug)]
struct FileStruct {
    message_array: Vec<Message>,
    game_mod: String,
}

impl FileStruct {
    pub fn new() -> FileStruct {
        FileStruct {
            message_array: vec![],
            game_mod: "".to_string(),
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct Message {
    message_type: MessageType,
    message_name_full: String,
    message_name_no_suffix: String,
    field_array: Vec<Field>,
    comment: String,
    res_message: Option<*const Message>,
}

impl Message {
    pub fn new() -> Message {
        Message {
            message_type: MessageType::Req,
            message_name_full: "".to_string(),
            message_name_no_suffix: "".to_string(),
            field_array: vec![],
            comment: "".to_string(),
            res_message: None,
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct Field {
    is_array: bool,
    field_type: String,
    field_name: String,
    comment: String,
}

impl Field {
    pub fn new() -> Field {
        Field {
            is_array: false,
            field_name: "".to_string(),
            field_type: "".to_string(),
            comment: "".to_string(),
        }
    }
}

fn parse_file(filepath: &str) -> std::io::Result<FileStruct> {
    // 可以使用lazy_static 来提高性能,但是这样会 无法代码提示.可以在程序稳定后进行替换
    //    lazy_static! {
    //        static ref RE: Regex = Regex::new("...").unwrap();
    //    }

    // 1.regex

    // 1 name
    // 2 {
    let re_message_define = Regex::new("^[ 	]*message *([a-zA-Z_]\\w*) *(\\{)?").unwrap();
    let re_left_brace = Regex::new(" *(\\{)").unwrap();
    let re_right_brace = Regex::new(" *(})").unwrap();
    let re_comment = Regex::new("^[ 	]*//(.*)[\r|\n]+").unwrap();
    let re_package = Regex::new("^[ 	]*package[ 	]*ds_client\\.([a-zA-Z_]\\w*)").unwrap();
    // 1 repeated
    // 2 type
    // 3 name
    // 4 comment
    let re_field =
        Regex::new("^[ 	]*(repeated)? *(\\w+) *([a-zA-Z_]\\w*) .*;(?://)? *(.*)").unwrap();

    // 2.FileStruct
    let mut file_struct: FileStruct = FileStruct::new();

    let mut state = State::Normal;

    let file = File::open(filepath)?;
    let mut buf_reader = BufReader::new(file);

    let mut comment = "".to_string();
    loop {
        let mut line = &mut String::new();
        let read_len = buf_reader.read_line(line);
        if read_len.unwrap() == 0 {
            break;
        }

        //parse comment
        if re_comment.is_match(line) {
            comment = re_comment.captures(line).unwrap()[1].to_string();
        }
        if re_package.is_match(line) {
            file_struct.game_mod = re_package.captures(line).unwrap()[1].to_string();
        }

        match state {
            State::Normal => {
                // 是否匹配is_match
                let is_match = re_message_define.is_match(line.as_str());
                if is_match {
                    state = State::MessageDefine;

                    file_struct.message_array.push(Message::new());
                    // 找到刚刚添加的元素
                    let mut message = file_struct.message_array.last_mut().unwrap();
                    message.comment = comment.to_string();

                    // 单个匹配  指的是找到第一个匹配. 与之对应的是 captures_iter ，捕获多个
                    let group = re_message_define.captures(line.as_str()).unwrap();
                    if let Some(message_name) = group.get(1) {
                        message.message_name_full = message_name.as_str().to_string();

                        if message_name.as_str().ends_with("_req") {
                            message.message_type = MessageType::Req;
                            message.message_name_no_suffix = message_name
                                .as_str()
                                .strip_suffix("_req")
                                .unwrap()
                                .to_string();
                        } else if message_name.as_str().ends_with("_rsp") {
                            message.message_type = MessageType::Rsp;
                            message.message_name_no_suffix = message_name
                                .as_str()
                                .strip_suffix("_rsp")
                                .unwrap()
                                .to_string();
                        } else if message_name.as_str().ends_with("_notify") {
                            message.message_type = MessageType::Notify;
                            message.message_name_no_suffix = message_name
                                .as_str()
                                .strip_suffix("_notify")
                                .unwrap()
                                .to_string();
                        }
                    }

                    // 花括号在同一行
                    if group.get(2).is_some() {
                        state = State::MessageScopeBegin;
                    }
                }
            }
            State::MessageDefine => {
                if re_left_brace.is_match(line.as_str()) {
                    state = State::MessageScopeBegin;
                }
            }
            State::MessageScopeBegin => {
                // parse scope end
                if re_right_brace.is_match(line.as_str()) {
                    state = State::Normal;
                    // let mut message = file_struct.message_array.last().unwrap();
                    // eprintln!("message = {:#?}", message);
                    continue;
                }

                // parse field
                if re_field.is_match(line.as_str()) {
                    // 找到刚刚添加的元素
                    let mut message = file_struct.message_array.last_mut().unwrap();
                    message.field_array.push(Field::new());

                    // 找到刚刚添加的元素
                    let mut field = message.field_array.last_mut().unwrap();
                    let group = re_field.captures(line.as_str()).unwrap();
                    //// 每个()捕获的内容都会被捕获，如果没有匹配到，就是none，否则就是some

                    if let Some(_) = group.get(1) {
                        field.is_array = true;
                    }
                    if let Some(value) = group.get(2) {
                        field.field_type = value.as_str().to_string();
                    }
                    if let Some(value) = group.get(3) {
                        field.field_name = value.as_str().to_string();
                    }
                    if let Some(value) = group.get(4) {
                        field.comment = value.as_str().to_string();
                    }

                    // eprintln!("field = {:#?}", field);
                }
            }
        }
    }

    // 3.遍历file_struct.message_array， 对每个req的message， 寻找对应的rsp message。需要双重循环同一个vector
    let vec = &mut file_struct.message_array;
    for i in 0..vec.len() {
        // let req_message = &mut vec[i];
        if vec[i].message_type == MessageType::Req {
            for j in 0..vec.len() {
                // let rsp_message = &vec[j];
                if vec[j].message_type == MessageType::Rsp
                    && vec[i].message_name_no_suffix.as_str()
                        == vec[j].message_name_no_suffix.as_str()
                {
                    vec[i].res_message = Some(&vec[j]);
                }
            }
        }
    }

    // println!("c2d_list.len() {}",c2d_list.len());
    // println!("d2c_list.len() {}",d2c_list.len());

    // eprintln!("file_struct = {:#?}", file_struct);
    Ok(file_struct)
}

fn write_lua_file(c2d_file: &str, d2c_file: &str, file_struct: FileStruct) -> std::io::Result<()> {
    // 4.将结果写入文件
    let c2d_list: Vec<&Message> = file_struct
        .message_array
        .iter()
        .filter(|x1| x1.message_type == MessageType::Req)
        .map(|x2| x2)
        .collect();

    let d2c_list: Vec<&Message> = file_struct
        .message_array
        .iter()
        .filter(|x1| x1.message_type == MessageType::Rsp || x1.message_type == MessageType::Notify)
        .map(|x2| x2)
        .collect();

    //c2d file gen
    if let Ok(mut file) = File::open(c2d_file) {
        println!("----open file {}----", c2d_file);
        // let mut buf_reader = BufReader::new(file);
        let all_text:&mut String =&mut String::new();
        file.read_to_string(all_text)?;


        for message in c2d_list {
            // let re_fun = Regex::new( format!("^[ \t]*function[ \t]*{}\\.([a-zA-Z_]\\w*)",message) "").unwrap();

        }

    } else {
        if let Ok(mut file) = File::create(c2d_file) {
            println!("----create file {}----", c2d_file);
            write!(file, "--auto generated--\n")?;

            let table_name = Path::new(c2d_file).file_stem().unwrap().to_str().unwrap();

            // table define
            write!(file, "{}", format!("local {} = {{\t}}\n\n", table_name))?;

            // functions
            for message in c2d_list {
                // comment
                write!(
                    file,
                    "{}",
                    format!("---{} {}\n", message.message_name_full, message.comment)
                )?;
                let s: Vec<String> = message
                    .field_array
                    .iter()
                    .map(|x| {
                        format!(
                            "---@param {} {} {}\n",
                            x.field_name, x.field_type, x.comment
                        )
                    })
                    .collect();
                let param_comment = s.join("");
                write!(file, "{}", param_comment)?;

                // params
                let s: Vec<String> = message
                    .field_array
                    .iter()
                    .map(|x| x.field_name.to_string())
                    .collect();
                let params = s.join(", ");

                // function
                write!(
                    file,
                    "{}",
                    format!(
                        "function {}.{}({})\n",
                        table_name, message.message_name_full, params
                    )
                )?;

                // print
                if s.len() > 0 {
                    let params_1 = s.join(":%s, ");
                    let params_2 = s.join(", ");
                    write!(
                        file,
                        "{}",
                        format!(
                            "\tprint(bWriteLog and string.format(\"{}.{} {}\",{}))\n",
                            table_name, message.message_name_full, params_1, params_2
                        )
                    )?;
                } else {
                    write!(
                        file,
                        "{}",
                        format!(
                            "\tprint(bWriteLog and string.format(\"{}.{} \"))\n",
                            table_name, message.message_name_full
                        )
                    )?;
                }

                // end
                write!(file, "{}", format!("end\n\n\n"))?;
            }

            write!(file, "{}", format!("\nreturn {}\n", table_name))?;
        }
    }

    println!("----");

    Ok(())
}
