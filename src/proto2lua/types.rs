
use std::fmt;


use std::path::{PathBuf};

fn sizeof_varint(v: u32) -> usize {
    match v {
        0x0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1F_FFFF => 3,
        0x20_0000..=0xFFF_FFFF => 4,
        _ => 5,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Syntax {
    Proto2,
    Proto3,
}

impl Default for Syntax {
    fn default() -> Syntax {
        Syntax::Proto2
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Frequency {
    Optional,
    Repeated,
    Required,
}

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct MessageIndex {
    indexes: Vec<usize>,
}

impl fmt::Debug for MessageIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_set().entries(self.indexes.iter()).finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct EnumIndex {
    msg_index: MessageIndex,
    index: usize,
}

#[warn(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Bool,
    Enum(EnumIndex),
    Fixed64,
    Sfixed64,
    Double,
    StringCow,
    BytesCow,
    String_,
    Bytes_,
    Message(MessageIndex),
    MessageOrEnum(String),
    Fixed32,
    Sfixed32,
    Float,
    Map(Box<FieldType>, Box<FieldType>),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub frequency: Frequency,
    pub typ: FieldType,
    pub number: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
    pub oneofs: Vec<OneOf>,
    pub msg_comment: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Enumerator {
    pub name: String,
    pub fields: Vec<(String, i32)>,
}

#[derive(Debug, Clone, Default)]
pub struct OneOf {
    pub name: String,
    pub fields: Vec<Field>,
    pub package: String,
    pub module: String,
    pub imported: bool,
}

#[derive(Debug, Default, Clone)]
pub struct FileDescriptor {
    pub import_paths: Vec<PathBuf>,
    pub package: String,
    pub syntax: Syntax,
    pub messages: Vec<Message>,
    pub enums: Vec<Enumerator>,
}
