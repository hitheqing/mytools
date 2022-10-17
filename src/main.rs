use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::windows::fs::FileExt;
use std::path::{Path, PathBuf};

use regex::Regex;

static mut COMMAND_LINE_OP: CommandLineOp = CommandLineOp {
    dir: String::new(),
    show_func_write: false,
};

// 第一个程序：读文件、正则匹配、文件结构parse、写文件

fn main() {
    // parse args
    let args: Vec<String> = std::env::args().collect();
    let mut dir = Path::new("");
    if args.get(1).is_some() {
        eprintln!("args.get(1).unwrap().as_str() = {:#?}", args.get(1).unwrap().as_str());
        dir = Path::new(args.get(1).unwrap().as_str());
    }

    let mut mod_dir = Path::new(".\\Mod");
    if args.get(2).is_some() {
        eprintln!("args.get(2).unwrap().as_str() = {:#?}", args.get(2).unwrap().as_str());
        mod_dir = Path::new(args.get(2).unwrap().as_str());
    }

    for x in &args {
        println!("---arg:{}", x);
        if x == "show_func_write" {
            unsafe {
                COMMAND_LINE_OP.show_func_write = true;
            }
        }
    }

    if let Ok(vec) = parse_dir(dir) {
        let client_suffix = "_Client_Handler";
        let ds_suffix = "_DS_Handler";

        for file_struct in &vec {
            if let Ok(_) = write_lua_file(mod_dir, client_suffix, ds_suffix, file_struct) {}
        }

        write_lua_route_config_file(mod_dir, &vec).expect("write route config failed");
    }
}

fn write_lua_route_config_file(mod_dir: &Path, file_struct_vec: &Vec<FileStruct>) -> std::io::Result<()> {
    if file_struct_vec.len() == 0 {
        return Ok(());
    }
    let mod_name = file_struct_vec.first().unwrap().mod_name.as_str();
    let filepath = Path::new(mod_dir).join(mod_name).join("GamePlay").join("Config").join("PBRouteConfig.lua");
    let dir = filepath.parent().unwrap();
    if false == dir.exists() {
        fs::create_dir_all(dir).expect("create failed");
    }
    if filepath.exists() {
        fs::remove_file(&filepath)?;
    }

    // 此文件每次删除重写
    if let Ok(mut file) = File::create(&filepath) {
        write!(file, "{}", format!("\n--- 各个Mod PB协议路由定义配置文件，此文件乃自动生成，请勿手动修改\n"))?;
        write!(file, "{}", format!("--- samizheng\n\n\n"))?;

        // client
        write!(file, "{}", format!("--1.PB协议：客户端响应DS的协议表\n"))?;
        write!(file, "{}", format!("local PBRouteConfig_Client =\n{{\n"))?;
        for item in file_struct_vec {
            write!(file, "{}", format!("\t{} =\n\t{{\n", item.file_name))?;
            let s = format!(
                "\t\tmoduleName = \"GameLua.Mod.{}.Client.Handler.{}_{}_Client_Handler\",\n",
                mod_name, mod_name, item.file_name
            );
            write!(file, "{}", s)?;
            // pbFileName = "SocialIsland/Alias.pb",
            write!(file, "{}", format!("\t\tpbFileName = \"{}/{}.pb\",\n", mod_name, item.file_name))?;
            write!(file, "{}", format!("\t\troutes =\n\t\t{{\n"))?;
            // aaa_rsp = "on_rsp"
            for msg in &item.messages {
                if msg.msg_type == MessageType::Req {
                    write!(file, "{}", format!("\t\t\t{} = true,\n", msg.msg_name_full))?;
                } else if msg.msg_type == MessageType::Rsp || msg.msg_type == MessageType::Notify {
                    write!(file, "{}", format!("\t\t\t{} = \"on_{}\",\n", msg.msg_name_full, msg.msg_name_full))?;
                }
            }
            write!(file, "{}", format!("\t\t}},\n"))?;
            write!(file, "{}", format!("\t}},\n"))?;
        }
        write!(file, "{}", format!("}}\n\n"))?;

        // ds
        write!(file, "{}", format!("--2.PB协议：DS响应客户端的协议表\n"))?;
        write!(file, "{}", format!("local PBRouteConfig_DS =\n{{\n"))?;
        for item in file_struct_vec {
            write!(file, "{}", format!("\t{} =\n\t{{\n", item.file_name))?;
            let s = format!(
                "\t\tmoduleName = \"GameLua.Mod.{}.DS.Handler.{}_{}_DS_Handler\",\n",
                mod_name, mod_name, item.file_name
            );
            write!(file, "{}", s)?;
            // pbFileName = "SocialIsland/Alias.pb",
            write!(file, "{}", format!("\t\tpbFileName = \"{}/{}.pb\",\n", mod_name, item.file_name))?;
            write!(file, "{}", format!("\t\troutes =\n\t\t{{\n"))?;
            // aaa_rsp = "on_rsp"
            for msg in &item.messages {
                if msg.msg_type == MessageType::Req {
                    write!(file, "{}", format!("\t\t\t{} = \"on_{}\",\n", msg.msg_name_full, msg.msg_name_full))?;
                } else if msg.msg_type == MessageType::Rsp || msg.msg_type == MessageType::Notify {
                    write!(file, "{}", format!("\t\t\t{} = true,\n", msg.msg_name_full))?;
                }
            }
            write!(file, "{}", format!("\t\t}},\n"))?;
            write!(file, "{}", format!("\t}},\n"))?;
        }
        write!(file, "{}", format!("}}\n\n\n"))?;

        //end
        write!(file, "{}", format!("if Client then\n"))?;
        write!(file, "{}", format!("\treturn PBRouteConfig_Client\n"))?;
        write!(file, "{}", format!("else\n"))?;
        write!(file, "{}", format!("\treturn PBRouteConfig_DS\n"))?;
        write!(file, "{}", format!("end\n"))?;
    }

    Ok(())
}

fn parse_dir(dir: &Path) -> std::io::Result<Vec<FileStruct>> {
    let mut result: Vec<FileStruct> = vec![];
    let dir = fs::read_dir(dir).unwrap();

    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let extension = path.extension().unwrap().to_str().unwrap();
            if extension == "proto" {
                if let Ok(fs) = parse_file(path.as_path()) {
                    result.push(fs)
                }
            }
        }
    }

    Ok(result)
}

/// 解析文件，得到文件结构
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
    // let re_package = Regex::new("^[ \t]*package[ \t]*ds_client\\.([a-zA-Z_]\\w*)").unwrap();
    // 1 repeated
    // 2 type
    // 3 name
    // 4 comment
    let re_field = Regex::new("^[ \t]*(repeated)?[ \t]*(\\w+)[ \t]*([a-zA-Z_]\\w*).*;[ \t]*(?://+)?(.*)").unwrap();

    // 2.FileStruct
    let mut file_struct: FileStruct = FileStruct::new();
    file_struct.file_name = filepath.file_stem().unwrap().to_str().unwrap().to_string();
    file_struct.mod_name = filepath.parent().unwrap().file_stem().unwrap().to_str().unwrap().to_string();

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
        // if re_package.is_match(line) {
        //     file_struct.game_mod = re_package.captures(line).unwrap()[1].to_string();
        // }

        match state {
            State::Normal => {
                // 是否匹配is_match
                let is_match = re_message_define.is_match(line.as_str());
                if is_match {
                    state = State::MessageDefine;

                    file_struct.messages.push(Message::new());
                    // 找到刚刚添加的元素
                    let mut message = file_struct.messages.last_mut().unwrap();
                    message.comment = comment.to_string();

                    // 单个匹配  指的是找到第一个匹配. 与之对应的是 captures_iter ，捕获多个
                    let group = re_message_define.captures(line.as_str()).unwrap();
                    if let Some(message_name) = group.get(1) {
                        message.msg_name_full = message_name.as_str().to_string();

                        if message_name.as_str().ends_with("_req") {
                            message.msg_type = MessageType::Req;
                            message.short_name = message_name.as_str().strip_suffix("_req").unwrap().to_string();
                        } else if message_name.as_str().ends_with("_rsp") {
                            message.msg_type = MessageType::Rsp;
                            message.short_name = message_name.as_str().strip_suffix("_rsp").unwrap().to_string();
                        } else if message_name.as_str().ends_with("_notify") {
                            message.msg_type = MessageType::Notify;
                            message.short_name = message_name.as_str().strip_suffix("_notify").unwrap().to_string();
                        } else {
                            message.msg_type = MessageType::Struct;
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
                    let message = file_struct.messages.last_mut().unwrap();
                    message.fields.push(Field::new());

                    // 找到刚刚添加的元素
                    let mut field = message.fields.last_mut().unwrap();
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
    let vec = &mut file_struct.messages;
    for i in 0..vec.len() {
        // let req_message = &mut vec[i];
        if vec[i].msg_type == MessageType::Req {
            for j in 0..vec.len() {
                // let rsp_message = &vec[j];
                if vec[j].msg_type == MessageType::Rsp && vec[i].short_name.as_str() == vec[j].short_name.as_str() {
                    vec[i].p_target = Some(&vec[j]);
                }
            }
        }
    }

    // eprintln!("file_struct = {:#?}", file_struct);
    Ok(file_struct)
}

/// 写入lua文件
fn write_lua_file(mod_dir: &Path, client_suffix: &str, ds_suffix: &str, file_struct: &FileStruct) -> std::io::Result<()> {
    // 4.将结果写入文件

    let c2d_list: Vec<&Message> = file_struct.messages.iter().filter(|x1| x1.msg_type == MessageType::Req).collect();

    let d2c_list: Vec<&Message> = file_struct
        .messages
        .iter()
        .filter(|x1| x1.msg_type == MessageType::Rsp || x1.msg_type == MessageType::Notify)
        .collect();

    let struct_list: Vec<&Message> = file_struct.messages.iter().filter(|x1| x1.msg_type == MessageType::Struct).collect();

    let mod_name = file_struct.mod_name.as_str();
    let ds_file_name = format!("{}_{}{}.lua", file_struct.mod_name, file_struct.file_name, ds_suffix);
    let ds_path = mod_dir.join(mod_name).join("DS").join("Handler").join(ds_file_name);
    let client_file_name = format!("{}_{}{}.lua", file_struct.mod_name, file_struct.file_name, client_suffix);
    let client_path = mod_dir.join(mod_name).join("Client").join("Handler").join(client_file_name);

    // &mut c2d_list 这些容器传进去，有可能被移除其中的元素（找到存在的会剔除）。然后在下面依然用到了，会被改变的。所以这里类型应该就是原始vector。
    create_or_update_file(
        Vec::from_iter(c2d_list.iter()),
        Vec::from_iter(d2c_list.iter()),
        Vec::from_iter(struct_list.iter()),
        &ds_path,
        file_struct,
        true,
    )?;

    // 需要注意的是  client file 的req rsp调换了顺序
    create_or_update_file(
        Vec::from_iter(d2c_list.iter()),
        Vec::from_iter(c2d_list.iter()),
        Vec::from_iter(struct_list.iter()),
        &client_path,
        file_struct,
        false,
    )?;

    Ok(())
}

/// 创建或更新文件
fn create_or_update_file(
    mut on_msg_list: Vec<&&Message>,
    mut send_msg_list: Vec<&&Message>,
    mut struct_msg_list: Vec<&&Message>,
    path: &PathBuf,
    file_struct: &FileStruct,
    is_ds: bool,
) -> std::io::Result<()> {
    let dir = path.parent().unwrap();
    if false == dir.exists() {
        fs::create_dir_all(dir).expect("create failed");
    }

    let table_name = path.file_stem().unwrap().to_str().unwrap();
    // file gen
    if let Ok(file) = File::open(path) {
        // 读文件，如果未找到函数，或者函数签名不一致，则新增函数
        let mut buf_reader = BufReader::new(file);

        // 1.构造正则 on_xx send_xx struct
        let mut re_list_on: Vec<Regex> = vec![];
        for message in on_msg_list.iter() {
            if is_ds {
                let res = format!("^[ \t]*function[ \t]*{}\\.on_{}\\(playerUid, message\\)", table_name, message.msg_name_full);
                re_list_on.push(Regex::new(res.as_str()).unwrap());
            } else {
                let res = format!("^[ \t]*function[ \t]*{}\\.on_{}\\(message\\)", table_name, message.msg_name_full);
                re_list_on.push(Regex::new(res.as_str()).unwrap());
            }
        }

        let mut re_list_send: Vec<Regex> = vec![];
        for message in send_msg_list.iter() {
            let params = message.get_params_str_in_func(is_ds, false);
            let res = format!("^[ \t]*function[ \t]*{}\\.send_{}\\({}\\)", table_name, message.msg_name_full, params);
            re_list_send.push(Regex::new(res.as_str()).unwrap());
        }

        let mut re_list_struct: Vec<Regex> = vec![];
        for message in struct_msg_list.iter() {
            let res = format!("^[ \t]*local[ \t]*{}", message.msg_name_full);
            re_list_struct.push(Regex::new(res.as_str()).unwrap());
        }

        // 2.遍历文件，剔除已经有了的结构。记录文件结尾的位置

        let mut pos = 0;
        let mut append_pos = 0;
        loop {
            let line = &mut String::new();
            let read_len = buf_reader.read_line(line);
            let last_pos = pos;
            pos = buf_reader.stream_position()?;
            if read_len.unwrap() == 0 {
                break;
            }

            let lambda = |re_list: &mut Vec<Regex>, msg_list: &mut Vec<&&Message>| {
                for i in (0..re_list.len()).rev() {
                    if re_list[i].is_match(line) {
                        // println!("exist func in {}", line);
                        re_list.remove(i);
                        msg_list.remove(i);
                        break;
                    }
                }
            };
            lambda(&mut re_list_on, &mut on_msg_list);
            lambda(&mut re_list_send, &mut send_msg_list);
            lambda(&mut re_list_struct, &mut struct_msg_list);

            // 找到return开始的地方，插入代码的地方
            if line.starts_with(format!("return {}", table_name).as_str()) {
                append_pos = last_pos;
            }
        }

        if on_msg_list.len() > 0 || send_msg_list.len() > 0 || struct_msg_list.len() > 0 {
            // 还剩下需要append的，加到后面
            let mut file = OpenOptions::new().write(true).open(path.as_path())?;
            println!("----update file {}----", path.as_path().to_str().unwrap());

            // 文件清空内容重新构造
            if append_pos == 0 {
                // table define
                write!(file, "{}", format!("local {} = {{\t}}\n", table_name))?;
                write!(file, "{}", format!("local ds_net = require(\"ds_net\")\n\n"))?;
            } else {
                //追加
                // 找到seek位置
                file.seek_write(b"-----autogen update below-----\n\n", append_pos)?;
            }

            // functions
            insert_function_code(on_msg_list, send_msg_list, struct_msg_list, &mut file, table_name, file_struct, is_ds)?;
            // return
            write!(file, "{}", format!("return {}\n", table_name))?;
        }
    } else {
        if let Ok(mut file) = File::create(path.as_path()) {
            println!("----create file {}----", path.as_path().to_str().unwrap());
            write!(file, "--auto generated--\n")?;

            // table define
            write!(file, "{}", format!("local {} = {{\t}}\n", table_name))?;
            write!(file, "{}", format!("local ds_net = require(\"ds_net\")\n\n"))?;

            // functions
            insert_function_code(on_msg_list, send_msg_list, struct_msg_list, &mut file, table_name, file_struct, is_ds)?;

            // return
            write!(file, "{}", format!("return {}\n", table_name))?;
        }
    }

    Ok(())
}

/// 插入函数代码
fn insert_function_code(
    on_msg_list: Vec<&&Message>,
    send_msg_list: Vec<&&Message>,
    struct_msg_list: Vec<&&Message>,
    file: &mut File,
    table_name: &str,
    file_struct: &FileStruct,
    is_ds: bool,
) -> std::io::Result<()> {
    // structs class hint
    for message in struct_msg_list {
        unsafe {
            if COMMAND_LINE_OP.show_func_write {
                println!("--func:{}", message.msg_name_full);
            }
        }

        // class define
        write!(file, "{}", format!("---@class {} {}\n", message.msg_name_full, message.comment))?;

        // local
        write!(file, "{}", format!("local {} = {{\n", message.msg_name_full))?;

        let s: Vec<String> = message
            .fields
            .iter()
            .map(|x| format!("\t---{} {}\t{} = {}, \n", x.get_type_string(), x.comment, x.field_name, x.get_default_value()))
            .collect();
        let param_comment = s.join("");
        // params
        write!(file, "{}", param_comment)?;
        // end
        write!(file, "{}", format!("}}\n\n"))?;
    }

    // on functions
    for message in on_msg_list {
        unsafe {
            if COMMAND_LINE_OP.show_func_write {
                println!("--func:{}", message.msg_name_full);
            }
        }
        // comment
        write!(file, "{}", format!("---{}\n", message.comment))?;
        let params_doc_comment_vec: Vec<String> = message
            .fields
            .iter()
            .map(|x| format!("---@param {} {} {}\n", x.field_name, x.get_type_string(), x.comment))
            .collect();
        let param_comment = params_doc_comment_vec.join("");
        write!(file, "{}", param_comment)?;

        // function
        if is_ds {
            let s = format!("function {}.on_{}(playerUid, message)\n", table_name, message.msg_name_full);
            write!(file, "{}", s)?;
        } else {
            write!(file, "{}", format!("function {}.on_{}(message)\n", table_name, message.msg_name_full))?;
        }

        // print
        if message.fields.len() > 0 {
            let s = format!(
                "\tprint(bWriteLog and string.format(\"{}.on_{} {}\"{}))\n",
                table_name,
                message.msg_name_full,
                message.get_print_params_format_str(is_ds),
                message.get_params_str_in_func_with_message(is_ds)
            );
            write!(file, "{}", s)?;
        } else {
            let s = format!("\tprint(bWriteLog and string.format(\"{}.{} \"))\n", table_name, message.msg_name_full);
            write!(file, "{}", s)?;
        }

        // rsp
        if let Some(ptr) = message.p_target {
            for target_message in &file_struct.messages {
                unsafe {
                    if target_message.msg_name_full == (*ptr).msg_name_full {
                        // println!("--find rsp:{} for req:{}", target_message.msg_name_full, message.msg_name_full);
                        for x in &target_message.fields {
                            write!(file, "{}", format!("\tlocal {} = {}\n", x.field_name, x.get_default_value()))?;
                        }

                        let s = format!(
                            "\t{}.send_{}({})\n",
                            table_name,
                            target_message.msg_name_full,
                            target_message.get_params_str_in_func(true, false)
                        );
                        write!(file, "{}", s)?;
                        break;
                    }
                }
            }
        }

        // end
        write!(file, "{}", format!("end\n\n"))?;
    }

    // send functions
    for message in send_msg_list {
        unsafe {
            if COMMAND_LINE_OP.show_func_write {
                println!("--func:{}", message.msg_name_full);
            }
        }
        // comment
        write!(file, "{}", format!("---{}\n", message.comment))?;
        let s: Vec<String> = message
            .fields
            .iter()
            .map(|x| format!("---@param {} {} {}\n", x.field_name, x.get_type_string(), x.comment))
            .collect();
        let param_comment = s.join("");
        write!(file, "{}", param_comment)?;

        // function
        let params = message.get_params_str_in_func(is_ds, false);
        let s = format!("function {}.send_{}({})\n", table_name, message.msg_name_full, params);
        write!(file, "{}", s)?;

        // print
        if message.fields.len() > 0 {
            let s1 = message.get_print_params_format_str(is_ds);
            let s2 = message.get_params_str_in_func(is_ds, true);
            let s = format!(
                "\tprint(bWriteLog and string.format(\"{}.send_{} {}\"{}))\n",
                table_name, message.msg_name_full, s1, s2
            );
            write!(file, "{}", s)?;
        } else {
            let s = format!("\tprint(bWriteLog and string.format(\"{}.send_{} \"))\n", table_name, message.msg_name_full);
            write!(file, "{}", s)?;
        }

        // content
        write!(file, "{}", format!("\tlocal res_param = {{\n"))?;
        // params
        let s: Vec<String> = message
            .fields
            .iter()
            .map(|x1| format!("\t\t{} = {},\n", x1.field_name, x1.field_name))
            .collect();
        let params = s.join("");
        write!(file, "{}", format!("{}", params))?;
        write!(file, "{}", format!("\t}}\n"))?;

        if is_ds {
            let s = format!(
                "\tds_net.SendMessage(\"{}.{}\", res_param, playerUid)\n",
                file_struct.mod_name, message.msg_name_full
            );
            write!(file, "{}", s)?;
        } else {
            let s = format!("\tds_net.SendMessage(\"{}.{}\", res_param)\n", file_struct.mod_name, message.msg_name_full);
            write!(file, "{}", s)?;
        }
        write!(file, "{}", format!("end\n\n"))?;
    }

    Ok(())
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
    messages: Vec<Message>,
    mod_name: String,
    file_name: String,
}

impl FileStruct {
    pub fn new() -> FileStruct {
        FileStruct {
            messages: vec![],
            mod_name: "".to_string(),
            file_name: "".to_string(),
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct Message {
    msg_type: MessageType,
    msg_name_full: String,
    short_name: String,
    fields: Vec<Field>,
    comment: String,
    p_target: Option<*const Message>,
}

impl Message {}

impl Message {
    pub fn new() -> Message {
        Message {
            msg_type: MessageType::Req,
            msg_name_full: "".to_string(),
            short_name: "".to_string(),
            fields: vec![],
            comment: "".to_string(),
            p_target: None,
        }
    }

    pub fn get_params_str_in_func(&self, is_ds: bool, is_need_prefix: bool) -> String {
        // params
        let field_names: Vec<String> = self.fields.iter().map(|x| x.field_name.to_string()).collect();
        let mut params = field_names.join(", ");

        if field_names.len() > 0 {
            if is_ds {
                params = format!("playerUid, {}", params);
            }
        } else {
            if is_ds {
                params = format!("playerUid");
            }
        }

        if is_need_prefix {
            params = format!(", {}", params);
        }
        params
    }

    pub fn get_params_str_in_func_with_message(&self, is_ds: bool) -> String {
        // params
        let field_names: Vec<String> = self.fields.iter().map(|x| x.field_name.to_string()).collect();
        let mut params = field_names.join(", message.");
        if field_names.len() > 0 {
            params = format!(", message.{}", params);
            if is_ds {
                params = format!(", playerUid{}", params);
            }
        } else {
            params = "".to_string();
            if is_ds {
                params = ", playerUid".to_string();
            }
        }
        params
    }

    pub(crate) fn get_print_params_format_str(&self, is_ds: bool) -> String {
        // params
        let s: Vec<String> = self.fields.iter().map(|x| x.field_name.to_string()).collect();
        let mut params_1 = s.join(":%s, ").as_str().to_owned() + ":%s";
        if is_ds {
            params_1 = format!("playerUid:%s, {}", params_1);
        }
        params_1
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
    fn new() -> Field {
        Field {
            is_array: false,
            field_name: "".to_string(),
            field_type: "".to_string(),
            comment: "".to_string(),
        }
    }

    fn get_type_string(&self) -> String {
        if self.is_array {
            format!("{}[]", self.field_type)
        } else {
            // self.field_type.to_owned()
            match self.field_type.as_str() {
                "int32" => "number".to_string(),
                "int64" => "number".to_string(),
                "string" => "".to_string(),
                "float" => "number".to_string(),
                _ => self.field_type.to_owned(),
            }
        }
    }

    fn get_default_value(&self) -> &str {
        if self.is_array {
            "{}"
        } else {
            match self.field_type.as_str() {
                "int32" => "0",
                "int64" => "0",
                "string" => "\"\"",
                "float" => "0",
                _ => "nil",
            }
        }
    }
}

struct CommandLineOp {
    dir: String,
    show_func_write: bool,
}
