use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::windows::fs::FileExt;
use std::path::{Path, PathBuf};
use std::fs;
use regex::Regex;

fn main() {
    // 第一个程序：读文件、正则匹配、文件结构parse、写文件
    let path = Path::new("./src/example.proto");
    let file_struct = parse_file(path).unwrap();

    let mod_dir = ".\\Mod";
    let client_suffix = "_Client_Handle";
    let ds_suffix = "_Ds_Handle";

    write_lua_file(mod_dir, client_suffix, ds_suffix, &file_struct).unwrap();
}

enum State {
    Normal,
    MessageDefine,
    MessageScopeBegin,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum MessageType {
    Struct,
    Req,
    Rsp,
    Notify,
}

#[derive(Debug)]
struct FileStruct {
    message_array: Vec<Message>,
    game_mod: String,
    file_name: String,
}

impl FileStruct {
    pub fn new() -> FileStruct {
        FileStruct {
            message_array: vec![],
            game_mod: "".to_string(),
            file_name: "".to_string(),
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

fn parse_file(filepath: &Path) -> std::io::Result<FileStruct> {
    // 可以使用lazy_static 来提高性能,但是这样会 无法代码提示.可以在程序稳定后进行替换
    //    lazy_static! {
    //        static ref RE: Regex = Regex::new("...").unwrap();
    //    }

    // 1.regex

    // 1 name
    // 2 {
    let re_message_define = Regex::new("^[ \t]*message[ \t]*([a-zA-Z_]\\w*)[ \t]*(\\{)?").unwrap();
    let re_left_brace = Regex::new("^[ \t]*(\\{)").unwrap();
    let re_right_brace = Regex::new("^[ \t]*(})").unwrap();
    let re_comment = Regex::new("^[ \t]*//(.*)[\r|\n]+").unwrap();
    let re_package = Regex::new("^[ \t]*package[ \t]*ds_client\\.([a-zA-Z_]\\w*)").unwrap();
    // 1 repeated
    // 2 type
    // 3 name
    // 4 comment
    let re_field =
        Regex::new("^[ \t]*(repeated)?[ \t]*(\\w+)[ \t]*([a-zA-Z_]\\w*).*;[ \t]*(?://+)?(.*)").unwrap();

    // 2.FileStruct
    let mut file_struct: FileStruct = FileStruct::new();
    file_struct.file_name = filepath.file_stem().unwrap().to_str().unwrap().to_string();

    let mut state = State::Normal;

    let file = File::open(filepath)?;
    let mut buf_reader = BufReader::new(file);

    let mut comment = "".to_string();
    loop {
        let line = &mut String::new();
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
                        } else {
                            message.message_type = MessageType::Struct;
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
                    let message = file_struct.message_array.last_mut().unwrap();
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

    // eprintln!("file_struct = {:#?}", file_struct);
    Ok(file_struct)
}

fn write_lua_file(mod_dir: &str, client_suffix: &str, ds_suffix: &str, file_struct: &FileStruct) -> std::io::Result<()> {
    // 4.将结果写入文件
    let mut c2d_list: Vec<&Message> = file_struct.message_array.iter().filter(|x1| x1.message_type == MessageType::Req).map(|x2| x2).collect();

    let mut d2c_list: Vec<&Message> = file_struct.message_array.iter().filter(|x1| x1.message_type == MessageType::Rsp || x1.message_type == MessageType::Notify).map(|x2| x2).collect();

    let mod_name = file_struct.game_mod.as_str();
    // GameLua.Mod.SocialIsland.Client.Handler.SocialIsland_Duel_Client_Handler
    let ds_file = Path::new(mod_dir).join(mod_name).join("DS").join("Handler").join(format!("{}_{}{}.lua", file_struct.game_mod, file_struct.file_name, ds_suffix));
    let client_file = Path::new(mod_dir).join(mod_name).join("Client").join("Handler").join(format!("{}_{}{}.lua", file_struct.game_mod, file_struct.file_name, client_suffix));

    println!("ds_file{:?}", ds_file);
    println!("client_file{:?}", client_file);


    create_or_update_file(&mut c2d_list, &mut d2c_list, &ds_file, file_struct, true)?;
    create_or_update_file(&mut d2c_list, &mut c2d_list, &client_file, file_struct, false)?;

    Ok(())
}

fn create_or_update_file(on_msg_list: &mut Vec<&Message>, send_msg_list: &mut Vec<&Message>, path: &PathBuf, file_struct: &FileStruct, is_ds: bool) -> std::io::Result<()> {
    let mut dir = path.parent().unwrap();
    if false == dir.exists() {
        fs::create_dir_all(dir).expect("create failed");
    }

    let table_name = path.file_stem().unwrap().to_str().unwrap();
    // file gen
    if let Ok(mut file) = File::open(path) {
        println!("----open file {}----", path.as_path().to_str().unwrap());

        // 读文件，如果未找到函数，或者函数签名不一致，则新增函数
        let mut buf_reader = BufReader::new(file);

        // 1.构造正则
        let mut re_vec: Vec<Regex> = vec![];
        for message in on_msg_list.iter() {
            // params
            let s: Vec<String> = message.field_array.iter().map(|x| x.field_name.to_string()).collect();
            let params = s.join(", ");

            let res = format!(
                "^[ \t]*function {}.{}\\({}\\)",
                table_name, message.message_name_full, params
            );
            let re_fun = Regex::new(res.as_str()).unwrap();
            re_vec.push(re_fun);
        }

        let mut last_pos = 0;
        let mut pos = 0;
        let mut append_pos = 0;
        loop {
            let line = &mut String::new();
            let read_len = buf_reader.read_line(line);
            last_pos = pos;
            pos = buf_reader.stream_position()?;
            if read_len.unwrap() == 0 {
                break;
            }

            for i in (0..re_vec.len()).rev() {
                if re_vec[i].is_match(line) {
                    // println!("exist func in {}", line);
                    re_vec.remove(i);
                    // 一次只可能匹配到一个，直接break
                    on_msg_list.remove(i);

                    break;
                }
            }

            // 找到return开始的地方，插入代码的地方
            if line.starts_with(format!("return {}", table_name).as_str()) {
                append_pos = last_pos;
            }
        }

        if on_msg_list.len() > 0 {
            // 还剩下需要append的，加到后面
            let mut file = OpenOptions::new().write(true).open(path.as_path())?;

            // 文件清空内容重新构造
            if append_pos == 0 {
                // table define
                write!(file, "{}", format!("local {} = {{\t}}\n\n", table_name))?;
            } else { //追加
                // 找到seek位置
                file.seek_write(b"-----autogen update below-----\n\n", append_pos)?;
            }

            // functions
            insert_function_code(&on_msg_list, &send_msg_list, &mut file, table_name, file_struct, is_ds)?;
            // return
            write!(file, "{}", format!("return {}\n", table_name))?;
        }
    } else {
        if let Ok(mut file) = File::create(path.as_path()) {
            println!("----create file {}----", path.as_path().to_str().unwrap());
            write!(file, "--auto generated--\n")?;

            // table define
            write!(file, "{}", format!("local {} = {{\t}}\n\n", table_name))?;

            // functions
            insert_function_code(&on_msg_list, &send_msg_list, &mut file, table_name, file_struct, is_ds)?;

            // return
            write!(file, "{}", format!("return {}\n", table_name))?;
        }
    }

    Ok(())
}

fn insert_function_code(
    on_msg_list: &Vec<&Message>,
    send_msg_list: &Vec<&Message>,
    file: &mut File,
    table_name: &str,
    file_struct: &FileStruct,
    is_ds: bool,
) -> std::io::Result<()> {
    // send functions
    for message in send_msg_list {
        // comment
        write!(
            file,
            "{}",
            format!("---{} {}\n", message.message_name_full, message.comment)
        )?;
        let s: Vec<String> = message.field_array
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
                "function {}.send_{}({})\n",
                table_name, message.message_name_full, params
            )
        )?;

        // print
        if s.len() > 0 {
            let params_1 = s.join(":%s, ").as_str().to_owned() + ":%s";
            let params_2 = s.join(", ");
            write!(
                file,
                "{}",
                format!(
                    "\tprint(bWriteLog and string.format(\"{}.send_{} {}\",{}))\n",
                    table_name, message.message_name_full, params_1, params_2
                )
            )?;
        } else {
            write!(
                file,
                "{}",
                format!(
                    "\tprint(bWriteLog and string.format(\"{}.send_{} \"))\n",
                    table_name, message.message_name_full
                )
            )?;
        }

        // content
        write!(file, "{}", format!("\tlocal res_param = {{\n"))?;
        // params
        let s: Vec<String> = message.field_array.iter().map(|x1| format!("\t\t{} = {},\n", x1.field_name, x1.field_name)).collect();
        let params = s.join("");
        write!(file, "{}", format!("{}", params))?;
        write!(file, "{}", format!("\t}}\n"))?;
        write!(file, "{}", format!("\tlocal ds_net = require(\"ds_net\")\n"))?;
        if is_ds {
            write!(file, "{}", format!("\tds_net.SendMessage(\"SocialIsland.{}\", res_param, playerUid)\n", message.message_name_full))?;
        } else {
            write!(file, "{}", format!("\tds_net.SendMessage(\"SocialIsland.{}\", res_param)\n", message.message_name_full))?;
        }
        write!(file, "{}", format!("end\n\n"))?;
    }

    // on functions
    for message in on_msg_list {
        // comment
        write!(
            file,
            "{}",
            format!("---{} {}\n", message.message_name_full, message.comment)
        )?;
        let params_doc_comment_vec: Vec<String> = message.field_array
            .iter()
            .map(|x| {
                format!(
                    "---@param {} {} {}\n",
                    x.field_name, x.field_type, x.comment
                )
            })
            .collect();
        let param_comment = params_doc_comment_vec.join("");
        write!(file, "{}", param_comment)?;

        // params
        let params_vec: Vec<String> = message
            .field_array
            .iter()
            .map(|x| x.field_name.to_string())
            .collect();
        let params = params_vec.join(", ");

        // function
        write!(
            file,
            "{}",
            format!(
                "function {}.on_{}(playerUid, message)\n",
                table_name, message.message_name_full
            )
        )?;

        // print
        if params_vec.len() > 0 {
            let params_1 = format!("{}{}", params_vec.join(":%s, "), ":%s");
            let params_2 = format!("{}{}", "message.", params_vec.join(", message."));
            write!(
                file,
                "{}",
                format!(
                    "\tprint(bWriteLog and string.format(\"{}.on_{} {}\",{}))\n",
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


        // rsp
        if let Some(ptr) = message.res_message {
            println!("-----{}", message.message_name_full);
            for x in &file_struct.message_array {
                unsafe {
                    if x.message_name_full == (*ptr).message_name_full {
                        write!(file, "{}", format!("\tlocal res_param = {{\n"))?;
                        // params
                        let s: Vec<String> = x.field_array.iter()
                            .map(|x1| format!("\t\t{} = {},\n", x1.field_name, x1.field_name))
                            .collect();
                        let params = s.join("");
                        write!(file, "{}", format!("{}", params))?;
                        write!(file, "{}", format!("\t}}\n"))?;
                        write!(file, "{}", format!("\tlocal ds_net = require(\"ds_net\")\n"))?;
                        write!(file, "{}", format!("\tds_net.SendMessage(\"SocialIsland.{}\", res_param, playerUid)\n", x.message_name_full))?;

                        break;
                    }
                }
            }
        }

        // end
        write!(file, "{}", format!("end\n\n"))?;
    }

    Ok(())
}
