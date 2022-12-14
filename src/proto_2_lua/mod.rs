use std::{
    mem::MaybeUninit,
    sync::{Mutex, Once},
};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, Write};
use std::os::windows::fs::FileExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use pb_rs::types::Config;
use regex::Regex;

use class_def::*;

use crate::{get_config, IDefault, init_config, Mode, MyApp};

mod class_def;


pub fn main(snake_case: MyApp) {
    let config = init_config(Some(snake_case));
    let client_suffix = "_Client_Handler";
    let ds_suffix = "_DS_Handler";

    if config.dir.is_none() {
        let mut result: Vec<FileStruct> = vec![];
        for file in &config.path {
            let path = Path::new(file.as_str());
            if path.is_file() {
                let extension = path.extension().unwrap().to_str().unwrap();
                if extension == "proto" {
                    if let Ok(fs) = parse_file(path) {
                        result.push(fs);
                    }
                }
            }
        }
        let mod_dir = Path::new(config.output.as_ref().unwrap().as_str());
        for file_struct in &result {
            if let Ok(_) = write_lua_file(mod_dir, client_suffix, ds_suffix, file_struct) {}
        }

        if config.write_route_config {
            write_lua_route_config_file(mod_dir, &result).expect("write route config failed");
        }
    } else {
        if let Some(dir) = &config.dir {
            if let Ok(vec) = parse_dir(Path::new(dir.as_str())) {
                let mod_dir = Path::new(config.output.as_ref().unwrap().as_str());
                for file_struct in &vec {
                    if let Ok(_) = write_lua_file(mod_dir, client_suffix, ds_suffix, file_struct) {}
                }

                if config.write_route_config {
                    write_lua_route_config_file(mod_dir, &vec).expect("write route config failed");
                }
            }
        }
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

    // ???????????????????????????
    if let Ok(mut file) = File::create(&filepath) {
        write!(file, "{}", format!("\n--- ??????Mod PB??????????????????????????????????????????????????????????????????????????????\n"))?;
        write!(file, "{}", format!("--- samizheng\n\n\n"))?;

        // client
        write!(file, "{}", format!("--1.PB????????????????????????DS????????????\n"))?;
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
        write!(file, "{}", format!("--2.PB?????????DS???????????????????????????\n"))?;
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
        let s = b"
if Client then
	return PBRouteConfig_Client
else
	return PBRouteConfig_DS
end
";
        file.write(s)?;
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

/// ?????????????????????????????????
fn parse_file(filepath: &Path) -> std::io::Result<FileStruct> {
    // ????????????lazy_static ???????????????,??????????????? ??????????????????.????????????????????????????????????
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
                // ????????????is_match
                let is_match = re_message_define.is_match(line.as_str());
                if is_match {
                    state = State::MessageDefine;

                    file_struct.messages.push(Message::new());
                    // ???????????????????????????
                    let mut message = file_struct.messages.last_mut().unwrap();
                    message.comment = comment.to_string();

                    // ????????????  ??????????????????????????????. ?????????????????? captures_iter ???????????????
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

                    // ?????????????????????
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
                    // ???????????????????????????
                    let message = file_struct.messages.last_mut().unwrap();
                    message.fields.push(Field::new());

                    // ???????????????????????????
                    let mut field = message.fields.last_mut().unwrap();
                    let group = re_field.captures(line.as_str()).unwrap();
                    //// ??????()???????????????????????????????????????????????????????????????none???????????????some

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

    // 3.??????file_struct.message_array??? ?????????req???message??? ???????????????rsp message??????????????????????????????vector
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

/// ??????lua??????
fn write_lua_file(mod_dir: &Path, client_suffix: &str, ds_suffix: &str, file_struct: &FileStruct) -> std::io::Result<()> {
    // 4.?????????????????????
    let mod_name = file_struct.mod_name.as_str();
    let ds_file_name = format!("{}_{}{}.lua", file_struct.mod_name, file_struct.file_name, ds_suffix);
    let ds_path = mod_dir.join(mod_name).join("DS").join("Handler").join(ds_file_name);
    let client_file_name = format!("{}_{}{}.lua", file_struct.mod_name, file_struct.file_name, client_suffix);
    let client_path = mod_dir.join(mod_name).join("Client").join("Handler").join(client_file_name);

    create_or_update_file(&ds_path, file_struct, true)?;
    create_or_update_file(&client_path, file_struct, false)?;

    Ok(())
}

/// ?????????????????????
fn create_or_update_file(path: &PathBuf, file_struct: &FileStruct, is_ds: bool) -> std::io::Result<()> {
    let dir = path.parent().unwrap();
    if false == dir.exists() {
        fs::create_dir_all(dir).expect("create failed");
    }

    let table_name = path.file_stem().unwrap().to_str().unwrap();
    // copy vector
    let mut remain_messages = Vec::from_iter(file_struct.messages.iter());

    // file gen
    if let Ok(file) = File::open(path) {
        // ?????????????????????????????????????????????????????????????????????????????????
        let mut buf_reader = BufReader::new(file);

        // 2.????????????????????????????????????????????????????????????????????????
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

            for i in (0..remain_messages.len()).rev() {
                let message = remain_messages[i];
                let re = message.make_func_regex(is_ds, table_name);
                if re.is_match(line) {
                    remain_messages.remove(i);
                }
            }

            // ??????return???????????????????????????????????????
            if line.starts_with(format!("return {}", table_name).as_str()) {
                append_pos = last_pos;
            }
        }

        if remain_messages.len() > 0 {
            // ???????????????append??????????????????
            let mut file = OpenOptions::new().write(true).open(path.as_path())?;

            if get_config().debug {
                println!("----update file {}----", path.as_path().to_str().unwrap());
            }

            // ??????????????????????????????
            if append_pos == 0 {
                // table define
                write!(file, "{}", format!("local {} = {{\t}}\n", table_name))?;
                write!(file, "{}", format!("local ds_net = require(\"ds_net\")\n\n"))?;
            } else {
                //??????
                // ??????seek??????
                file.seek_write(b"-----autogen update below-----\n\n", append_pos)?;
            }

            // functions
            insert_function_code(remain_messages, &mut file, table_name, file_struct, is_ds)?;
            // return
            write!(file, "{}", format!("return {}\n", table_name))?;
        } else {
            // println!("----keep file {}----", path.as_path().to_str().unwrap());
        }
    } else {
        if let Ok(mut file) = File::create(path.as_path()) {
            if get_config().debug {
                println!("----create file {}----", path.as_path().to_str().unwrap());
            }

            write!(file, "--auto generated--\n")?;

            // table define
            write!(file, "{}", format!("local {} = {{\t}}\n", table_name))?;
            write!(file, "{}", format!("local ds_net = require(\"ds_net\")\n\n"))?;

            // functions
            insert_function_code(remain_messages, &mut file, table_name, file_struct, is_ds)?;

            // return
            write!(file, "{}", format!("return {}\n", table_name))?;
        }
    }

    Ok(())
}

/// ??????????????????
fn insert_function_code(remain_messages: Vec<&Message>, file: &mut File, table_name: &str, file_struct: &FileStruct, is_ds: bool) -> std::io::Result<()> {
    for message in remain_messages {


        if MessageType::Struct == message.msg_type {
            // class define
            write!(file, "{}", message.gen_class_doc_comment(""))?;
            write!(file, "{}", message.gen_table_string("--", None, true))?;
            if get_config().debug {
                println!("--class:{}", message.msg_name_full);
            }
        } else if MessageType::Req == message.msg_type {
            // function doc
            write!(file, "{}", message.gen_func_doc_comment(""))?;
            // function define
            write!(file, "{}", message.gen_func_string(is_ds, table_name))?;
            // print
            if is_ds {
                if message.fields.len() > 0 {
                    let s = format!(
                        "\tprint(bWriteLog and string.format(\"{}.on_{} {}\"{}))\n",
                        table_name,
                        message.msg_name_full,
                        message.get_print_params_format_str(is_ds),
                        message.get_params_str_in_func_with_message(is_ds, true)
                    );
                    write!(file, "{}", s)?;
                } else {
                    let s = format!(
                        "\tprint(bWriteLog and string.format(\"{}.{} playerUid:%s\",playerUid))\n",
                        table_name, message.msg_name_full
                    );
                    write!(file, "{}", s)?;
                }

                if get_config().debug {
                    println!("--func:on_{}", message.msg_name_full);
                }
            } else {
                if message.fields.len() > 0 {
                    let s = format!(
                        "\tprint(bWriteLog and string.format(\"{}.send_{} {}\"{}))\n",
                        table_name,
                        message.msg_name_full,
                        message.get_print_params_format_str(is_ds),
                        message.get_params_str_in_func_with_message(is_ds, false)
                    );
                    write!(file, "{}", s)?;
                } else {
                    let s = format!("\tprint(bWriteLog and string.format(\"{}.{} \"))\n", table_name, message.msg_name_full);
                    write!(file, "{}", s)?;
                }
                if get_config().debug {
                    println!("--func:send_{}", message.msg_name_full);
                }
            }
            // rsp ds only
            if is_ds {
                if let Some(ptr) = message.p_target {
                    for target_message in &file_struct.messages {
                        unsafe {
                            if target_message.msg_name_full == (*ptr).msg_name_full {
                                // println!("--find rsp:{} for req:{}", target_message.msg_name_full, message.msg_name_full);
                                for x in &target_message.fields {
                                    write!(file, "{}", format!("\tlocal {} = {}\n", x.field_name, x.get_default_value()))?;
                                }

                                let s = format!(
                                    "\t{}.{}({})\n",
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
            } else {
                // content
                write!(file, "{}", message.gen_table_string("\t", Some("res_param"), false))?;
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
            }

            // end
            write!(file, "{}", format!("end\n\n"))?;
        } else if MessageType::Rsp == message.msg_type || MessageType::Notify == message.msg_type {
            // function doc
            write!(file, "{}", message.gen_func_doc_comment(""))?;
            // function define
            write!(file, "{}", message.gen_func_string(is_ds, table_name))?;
            // print
            if is_ds {
                if message.fields.len() > 0 {
                    let s = format!(
                        "\tprint(bWriteLog and string.format(\"{}.{} {}\"{}))\n",
                        table_name,
                        message.msg_name_full,
                        message.get_print_params_format_str(is_ds),
                        message.get_params_str_in_func_with_message(is_ds, false)
                    );
                    write!(file, "{}", s)?;
                } else {
                    let s = format!("\tprint(bWriteLog and string.format(\"{}.{} \"))\n", table_name, message.msg_name_full);
                    write!(file, "{}", s)?;
                }
                if get_config().debug {
                    println!("--func:{}", message.msg_name_full);
                }
            } else {
                if message.fields.len() > 0 {
                    let s = format!(
                        "\tprint(bWriteLog and string.format(\"{}.on_{} {}\"{}))\n",
                        table_name,
                        message.msg_name_full,
                        message.get_print_params_format_str(is_ds),
                        message.get_params_str_in_func_with_message(is_ds, true)
                    );
                    write!(file, "{}", s)?;
                } else {
                    let s = format!("\tprint(bWriteLog and string.format(\"{}.{} \"))\n", table_name, message.msg_name_full);
                    write!(file, "{}", s)?;
                }
                if get_config().debug {
                    println!("--func:on_{}", message.msg_name_full);
                }
            }

            if is_ds {
                // content
                write!(file, "{}", message.gen_table_string("\t", Some("res_param"), false))?;

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
            }
            // end
            write!(file, "{}", format!("end\n\n"))?;
        }
    }

    Ok(())
}
