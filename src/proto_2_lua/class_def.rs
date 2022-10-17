use regex::Regex;

pub enum State {
    Normal,
    MessageDefine,
    MessageScopeBegin,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum MessageType {
    Struct,
    Req,
    Rsp,
    Notify,
}

#[derive(Debug)]
pub struct FileStruct {
    pub messages: Vec<Message>,
    pub mod_name: String,
    pub file_name: String,
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
pub struct Message {
    pub msg_type: MessageType,
    pub msg_name_full: String,
    pub short_name: String,
    pub fields: Vec<Field>,
    pub comment: String,
    pub p_target: Option<*const Message>,
}

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

    // is_need_prefix print中才传true
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

    pub fn get_params_str_in_func_with_message(&self, need_player_uid: bool, need_message: bool) -> String {
        // params
        let field_names: Vec<String> = self.fields.iter().map(|x| x.field_name.to_string()).collect();
        let mut params;
        if need_message {
            params = field_names.join(", message.");
        } else {
            params = field_names.join(", ");
        }

        if field_names.len() > 0 {
            if need_message && !need_player_uid {
                params = format!(", message.{}", params);
            } else if !need_message && need_player_uid {
                params = format!(", playerUid, {}", params);
            } else if need_message && need_player_uid {
                params = format!(", playerUid, message.{}", params);
            } else {
                params = format!(", {}", params);
            }
        } else {
            params = "".to_string();
            if need_player_uid {
                params = ", playerUid".to_string();
            }
        }
        params
    }

    pub(crate) fn get_print_params_format_str(&self, need_player_uid: bool) -> String {
        // params
        let s: Vec<String> = self.fields.iter().map(|x| x.field_name.to_string()).collect();
        let mut params_1 = s.join(":%s, ").as_str().to_owned() + ":%s";
        if need_player_uid {
            params_1 = format!("playerUid:%s, {}", params_1);
        }
        params_1
    }

    pub fn gen_func_doc_comment(&self, prefix: &str) -> String {
        let mut ret = String::new();
        let s = format!("---{}\n", self.comment);
        let s = format!("{}{}", prefix, s);
        ret.push_str(s.as_str());
        for x in &self.fields {
            let s = format!("---@param {} {} {}\n", x.field_name, x.get_type_string(), x.comment);
            let s = format!("{}{}", prefix, s);
            ret.push_str(s.as_str());
        }
        ret
    }

    pub fn gen_class_doc_comment(&self, prefix: &str) -> String {
        let mut ret = String::new();
        let s = format!("---@class {}\n", self.msg_name_full);
        ret.push_str(format!("{}{}", prefix, s).as_str());
        for x in &self.fields {
            let s = format!("---@field {} {} {}\n", x.field_name, x.get_type_string(), x.comment);
            ret.push_str(format!("{}{}", prefix, s).as_str());
        }
        ret
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `prefix`: 前缀。 如果注释 --  如果tab \t
    /// * `new_table_name`: 变量名 。none则使用message本身
    /// * `use_value`: 使用默认值， 否则使用 field变量名本身
    ///
    /// returns: String
    pub fn gen_table_string(&self, prefix: &str, new_table_name: Option<&str>, use_value: bool) -> String {
        let mut ret = String::new();

        // local
        let s = match new_table_name {
            None => {
                format!("local {} = {{\n", self.msg_name_full)
            }
            Some(table_name) => {
                format!("local {} = {{\n", table_name)
            }
        };
        // let s = format!("local {} = {{\n", self.msg_name_full);
        ret.push_str(format!("{}{}", prefix, s).as_str());
        for x in &self.fields {
            let s = if use_value { x.get_default_value() } else { x.field_name.as_str() };
            let s = format!("\t{} = {},\n", x.field_name, s);
            ret.push_str(format!("{}{}", prefix, s).as_str());
        }
        let s = format!("}}\n\n");
        ret.push_str(format!("{}{}", prefix, s).as_str());
        ret
    }

    pub fn gen_func_string(&self, is_ds: bool, table_name: &str) -> String {
        let mut s: String = String::new();
        let params = self.get_params_str_in_func(is_ds, false);
        match self.msg_type {
            MessageType::Struct => {
                "".to_string();
            }
            MessageType::Req => {
                if is_ds {
                    let ts = format!("function {}.on_{}(playerUid, message)\n", table_name, self.msg_name_full);
                    s.push_str(ts.as_str());
                } else {
                    let ts = format!("function {}.send_{}({})\n", table_name, self.msg_name_full, params);
                    s.push_str(ts.as_str());
                }
            }
            MessageType::Rsp => {
                if is_ds {
                    let ts = format!("function {}.send_{}({})\n", table_name, self.msg_name_full, params);
                    s.push_str(ts.as_str());
                } else {
                    let ts = format!("function {}.on_{}(message)\n", table_name, self.msg_name_full);
                    s.push_str(ts.as_str());
                }
            }
            MessageType::Notify => {
                if is_ds {
                    let ts = format!("function {}.send_{}({})\n", table_name, self.msg_name_full, params);
                    s.push_str(ts.as_str());
                } else {
                    let ts = format!("function {}.on_{}(message)\n", table_name, self.msg_name_full);
                    s.push_str(ts.as_str());
                }
            }
        }
        s
    }

    pub fn make_func_regex(&self, is_ds: bool, table_name: &str) -> Regex {
        let params = self.get_params_str_in_func(is_ds, false);
        match self.msg_type {
            MessageType::Struct => {
                let re = format!("^[ \t]*---@class[ \t]*{}", self.msg_name_full);
                Regex::new(re.as_str()).unwrap()
            }
            MessageType::Req => {
                if is_ds {
                    let re = format!("^[ \t]*function[ \t]*{}\\.on_{}\\(playerUid, message\\)", table_name, self.msg_name_full);
                    Regex::new(re.as_str()).unwrap()
                } else {
                    let re = format!("^[ \t]*function[ \t]*{}\\.send_{}\\({}\\)", table_name, self.msg_name_full, params);
                    Regex::new(re.as_str()).unwrap()
                }
            }
            MessageType::Rsp => {
                if is_ds {
                    let re = format!("^[ \t]*function[ \t]*{}\\.send_{}\\({}\\)", table_name, self.msg_name_full, params);
                    Regex::new(re.as_str()).unwrap()
                } else {
                    let re = format!("^[ \t]*function[ \t]*{}\\.on_{}\\(message\\)", table_name, self.msg_name_full);
                    Regex::new(re.as_str()).unwrap()
                }
            }
            MessageType::Notify => {
                if is_ds {
                    let re = format!("^[ \t]*function[ \t]*{}\\.send_{}\\({}\\)", table_name, self.msg_name_full, params);
                    Regex::new(re.as_str()).unwrap()
                } else {
                    let re = format!("^[ \t]*function[ \t]*{}\\.on_{}\\(message\\)", table_name, self.msg_name_full);
                    Regex::new(re.as_str()).unwrap()
                }
            }
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Field {
    pub is_array: bool,
    pub field_type: String,
    pub field_name: String,
    pub comment: String,
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

    pub(crate) fn get_default_value(&self) -> &str {
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
