use std::path::{Path, PathBuf};
use std::str;

use nom::{digit, hex_digit, multispace, space};
use nom::IResult::Error;
use walkdir::WalkDir;

use super::types::{Enumerator, Field, FieldType, FileDescriptor, Frequency, Message, OneOf, Syntax};

fn is_word(b: u8) -> bool {
    match b {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.' => true,
        _ => false,
    }
}

named!(word<String>, map_res!(take_while!(is_word), |b: &[u8]| String::from_utf8(b.to_vec())));
named!(word_ref<&str>, map_res!(take_while!(is_word), str::from_utf8));

named!(
    hex_integer<i32>,
    do_parse!(tag!("0x") >> num: map_res!(map_res!(hex_digit, str::from_utf8), |s| i32::from_str_radix(s, 16)) >> (num))
);

named!(integer<i32>, map_res!(map_res!(digit, str::from_utf8), str::FromStr::from_str));

named!(comment<()>, do_parse!(tag!("//") >> take_until_and_consume!("\n") >> ()));
named!(block_comment<()>, do_parse!(tag!("/*") >> take_until_and_consume!("*/") >> ()));

// 多个 /////
named!(
    take_comment<String>,
    do_parse!(tag!("//") >> many0!(tag!("/")) >> content: take_until_and_consume!("\n") >> (String::from_utf8(content.to_vec()).unwrap()))
);
named!(
    take_block_comment<String>,
    do_parse!(tag!("/*") >> content: take_until_and_consume!("*/") >> (String::from_utf8(content.to_vec()).unwrap()))
);

// word break: multispace or comment
named!(
    br<()>,
    alt!(map!(multispace, |_| ()) | comment | block_comment) // alt!(map!(multispace, |_| ()))
);

// multispace or comment
named!(
    br_or_comment<()>,
    alt!(map!(multispace, |_| ()) | comment | block_comment) // alt!(map!(multispace, |_| ()))
);

named!(take_valid_comment<String>, alt!(take_comment | take_block_comment));

// we need comment
named!(
    comment2<String>,
    do_parse!(tag!("//") >> content: take_until!("\n") >> (String::from_utf8(content.to_vec()).unwrap_or("".to_string())))
);

named!(
    syntax<Syntax>,
    do_parse!(
        tag!("syntax")
            >> many0!(br)
            >> tag!("=")
            >> many0!(br)
            >> proto:
                alt!(tag!("\"proto2\"") => { |_| Syntax::Proto2 } |
                             tag!("\"proto3\"") => { |_| Syntax::Proto3 })
            >> many0!(br)
            >> tag!(";")
            >> (proto)
    )
);

named!(
    import<PathBuf>,
    do_parse!(
        tag!("import")
            >> many1!(br)
            >> tag!("\"")
            >> path: map!(map_res!(take_until!("\""), str::from_utf8), |s| Path::new(s).into())
            >> tag!("\"")
            >> many0!(br)
            >> tag!(";")
            >> (path)
    )
);

named!(
    package<String>,
    do_parse!(tag!("package") >> many1!(br) >> package: word >> many0!(br) >> tag!(";") >> (package))
);

named!(extensions<()>, do_parse!(tag!("extensions") >> take_until_and_consume!(";") >> ()));

named!(
    frequency<Frequency>,
    alt!(tag!("optional") => { |_| Frequency::Optional } |
            tag!("repeated") => { |_| Frequency::Repeated } |
            tag!("required") => { |_| Frequency::Required } )
);

named!(
    field_type<FieldType>,
    alt!(tag!("int32") => { |_| FieldType::Int32 } |
            tag!("int64") => { |_| FieldType::Int64 } |
            tag!("uint32") => { |_| FieldType::Uint32 } |
            tag!("uint64") => { |_| FieldType::Uint64 } |
            tag!("sint32") => { |_| FieldType::Sint32 } |
            tag!("sint64") => { |_| FieldType::Sint64 } |
            tag!("fixed32") => { |_| FieldType::Fixed32 } |
            tag!("sfixed32") => { |_| FieldType::Sfixed32 } |
            tag!("fixed64") => { |_| FieldType::Fixed64 } |
            tag!("sfixed64") => { |_| FieldType::Sfixed64 } |
            tag!("bool") => { |_| FieldType::Bool } |
            tag!("string") => { |_| FieldType::StringCow } |
            tag!("bytes") => { |_| FieldType::BytesCow } |
            tag!("float") => { |_| FieldType::Float } |
            tag!("double") => { |_| FieldType::Double } |
            map_field => { |(k, v)| FieldType::Map(Box::new(k), Box::new(v)) } |
            word => { |w| FieldType::MessageOrEnum(w) })
);

named!(
    map_field<(FieldType, FieldType)>,
    do_parse!(
        tag!("map")
            >> many0!(br)
            >> tag!("<")
            >> many0!(br)
            >> key: field_type
            >> many0!(br)
            >> tag!(",")
            >> many0!(br)
            >> value: field_type
            >> tag!(">")
            >> ((key, value))
    )
);

named!(
    one_of<OneOf>,
    do_parse!(
        tag!("oneof")
            >> many1!(br)
            >> name: word
            >> many0!(br)
            >> tag!("{")
            >> fields: many1!(message_field)
            >> many0!(br)
            >> tag!("}")
            >> many0!(br)
            >> (OneOf {
                name: name,
                fields: fields,
                package: "".to_string(),
                module: "".to_string(),
                imported: false,
            })
    )
);

named!(
    message_field<Field>,
    do_parse!(
        frequency: opt!(frequency)
            >> many0!(br)
            >> typ: field_type
            >> many1!(br)
            >> name: word
            >> many0!(br)
            >> tag!("=")
            >> many0!(br)
            >> number: alt!(hex_integer | integer)
            >> many0!(br)
            >> tag!(";")
            >> many0!(space)
            >> comment: opt!(take_comment)
            >> (Field {
                name: name,
                frequency: frequency.unwrap_or(Frequency::Optional),
                number: number,
                typ: typ,
                comment: comment,
            })
    )
);

enum MessageEvent {
    Field(Field),
    OneOf(OneOf),
    Ignore,
}

// nested message.
named!(
    message_event<MessageEvent>,
    alt!(
        message_field => { |f| MessageEvent::Field(f) } |
        one_of => { |o| MessageEvent::OneOf(o) } |
        extensions => { |_| MessageEvent::Ignore } |
        br => { |_| MessageEvent::Ignore })
);

named!(
    message_events<(String, Option<String>, Vec<MessageEvent>)>,
    do_parse!(
        comment: message_begin
            >> many1!(br)
            >> name: word
            >> many0!(br)
            >> tag!("{")
            >> many0!(br)
            >> events: many0!(message_event)
            >> many0!(br)
            >> tag!("}")
            >> many0!(multispace)
            >> many0!(tag!(";"))
            >> ((name, Some(comment), events))
    )
);

// parse 带有注释的message. 注释可能没有，所以是 option. 然后匹配应该就近原则，尽量紧凑
named!(
    message_begin<String>,
    do_parse!(comment: opt!(take_comment) >> many0!(space) >> tag!("message") >> (comment.unwrap_or("".to_string())))
);

named!(
    message<Message>,
    map!(message_events, |(name, msg_comment, events): (String, Option<String>, Vec<MessageEvent>)| {
        let mut msg = Message {
            name,
            msg_comment,
            ..Message::default()
        };
        for e in events {
            match e {
                MessageEvent::Field(f) => msg.fields.push(f),
                MessageEvent::OneOf(o) => msg.oneofs.push(o),
                MessageEvent::Ignore => (),
            }
        }
        msg
    })
);

named!(
    enum_field<(String, i32)>,
    do_parse!(
        name: word >> many0!(br) >> tag!("=") >> many0!(br) >> number: alt!(hex_integer | integer) >> many0!(br) >> tag!(";") >> many0!(br) >> ((name, number))
    )
);

named!(
    enumerator<Enumerator>,
    do_parse!(
        tag!("enum")
            >> many1!(br)
            >> name: word
            >> many0!(br)
            >> tag!("{")
            >> many0!(br)
            >> fields: many0!(enum_field)
            >> many0!(br)
            >> tag!("}")
            >> many0!(multispace)
            >> many0!(tag!(";"))
            >> (Enumerator {
                name: name,
                fields: fields,
                ..Enumerator::default()
            })
    )
);

named!(option_ignore<()>, do_parse!(tag!("option") >> many1!(br) >> take_until_and_consume!(";") >> ()));

enum Event {
    Syntax(Syntax),
    Import(PathBuf),
    Package(String),
    Message(Message),
    Enum(Enumerator),
    Ignore,
}

named!(
    event<Event>,
    alt!(syntax => { |s| Event::Syntax(s) } |
            import => { |i| Event::Import(i) } |
            package => { |p| Event::Package(p) } |
            message => { |m| Event::Message(m) } |
            enumerator => { |e| Event::Enum(e) } |
            option_ignore => { |_| Event::Ignore } |
            br => { |_| Event::Ignore }
    )
);

named!(pub file_descriptor<FileDescriptor>,
map!(many0!(event), |events: Vec<Event>| {
    let mut desc = FileDescriptor::default();
    let mut ct = 0;
    let len = events.len();
    for event in events {
        match event {
            Event::Syntax(s) => desc.syntax = s,
            Event::Import(i) => desc.import_paths.push(i),
            Event::Package(p) => desc.package = p,
            Event::Message(m) => desc.messages.push(m),
            Event::Enum(e) => desc.enums.push(e),
            Event::Ignore => (ct += 1),
        };
    }
    desc.valid_event_count = len - ct;
    desc
}));

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Read;
    use nom::{IError, IResult};

    use super::*;

    #[test]
    fn test_external_dir() {
        let dir = Path::new("../../Lua/proto/ds_client/SocialIsland");
        let proto_iter = WalkDir::new(dir)
            .into_iter()
            .filter_map(|x| x.ok())
            .filter(|x1| x1.path().extension().unwrap_or_default() == "proto");

        for entry in proto_iter {
            if let Ok(mut f) = File::open(entry.path()) {
                let buf: &mut Vec<u8> = &mut vec![];
                f.read_to_end(buf).unwrap();
                match file_descriptor(buf) {
                    IResult::Done(_, desc) => {
                        assert!(
                            desc.valid_event_count >= 3,
                            "may be syntax error.\npath:{}",
                            entry.path().to_str().unwrap()
                        );
                    }
                    Error(_) => {
                        assert!(false);
                    }
                    IResult::Incomplete(_) => {
                        assert!(false);
                    }
                }
            }
        }
    }

    #[test]
    fn test_message() {
        let msg = r#"message ReferenceData
    {
        repeated ScenarioInfo  scenarioSet = 1;// 注释啊哈哈
        repeated CalculatedObjectInfo calculatedObjectSet = 2;
    }"#;

        let mess = message(msg.as_bytes());
        if let IResult::Done(_, mess) = mess {
            eprintln!("mess = {:#?}", mess);
            assert_eq!(2, mess.fields.len());
        }
    }

    #[test]
    fn test_enum() {
        let msg = r#"enum PairingStatus {
                DEALPAIRED        = 0;
                INVENTORYORPHAN   = 1;
                CALCULATEDORPHAN  = 2;
                CANCELED          = 3;
    }"#;

        let mess = enumerator(msg.as_bytes());
        if let IResult::Done(_, mess) = mess {
            assert_eq!(4, mess.fields.len());
        }
    }

    #[test]
    fn test_take_comment() {
        let msg = r#"

        //aaa
        dfsdf
        "#;
        let mess = take_comment(msg.as_bytes());
        if let IResult::Done(_, mess) = mess {
            println!("mess = {}", mess);
            assert_eq!("aaa2".to_string(), mess);
        }
    }

    #[test]
    fn test_from_file() {
        let mut s = File::open("./src/Duel.proto").unwrap();
        let buf: &mut Vec<u8> = &mut vec![];
        s.read_to_end(buf).unwrap();
        let desc = file_descriptor(buf).to_full_result().unwrap();
        // assert_eq!("foo.bar".to_string(), desc.package);
        eprintln!("desc = {:#?}", desc);
    }

    #[test]
    fn test_ignore() {
        let msg = r#"option optimize_for = SPEED;"#;

        match option_ignore(msg.as_bytes()) {
            IResult::Done(_, _) => (),
            e => panic!("Expecting done {:?}", e),
        }
    }

    #[test]
    fn test_import() {
        let msg = r#"syntax = "proto3";

    import "test_import_nested_imported_pb.proto";

    message ContainsImportedNested {
        optional ContainerForNested.NestedMessage m = 1;
        optional ContainerForNested.NestedEnum e = 2;
    }
    "#;
        let desc = file_descriptor(msg.as_bytes()).to_full_result().unwrap();
        assert_eq!(vec![Path::new("test_import_nested_imported_pb.proto")], desc.import_paths);
    }

    #[test]
    fn test_package() {
        let msg = r#"
        package foo.bar;

//没用的注释1

//有用的注释2
    message ContainsImportedNested {
        optional ContainerForNested.NestedMessage m = 1;
        optional ContainerForNested.NestedEnum e = 2;
    }
    "#;
        let desc = file_descriptor(msg.as_bytes()).to_full_result().unwrap();
        assert_eq!("foo.bar".to_string(), desc.package);
        eprintln!("desc = {:#?}", desc);
    }

    #[test]
    fn test_map() {
        let msg = r#"message A
    {
        optional map<string, int32> b = 1;
    }"#;

        let mess = message(msg.as_bytes());
        if let IResult::Done(_, mess) = mess {
            assert_eq!(1, mess.fields.len());
            match mess.fields[0].typ {
                FieldType::Map(ref key, ref value) => match (&**key, &**value) {
                    (&FieldType::String_, &FieldType::Int32) => (),
                    (&FieldType::StringCow, &FieldType::Int32) => (),
                    _ => panic!("Expecting Map<String, Int32> found Map<{:?}, {:?}>", key, value),
                },
                ref f => panic!("Expecting map, got {:?}", f),
            }
        } else {
            panic!("Could not parse map message");
        }
    }

    #[test]
    fn test_oneof() {
        let msg = r#"message A
    {
        optional int32 a1 = 1;
        oneof a_oneof {
            string a2 = 2;
            int32 a3 = 3;
            bytes a4 = 4;
        }
        repeated bool a5 = 5;
    }"#;

        let mess = message(msg.as_bytes());
        if let IResult::Done(_, mess) = mess {
            assert_eq!(1, mess.oneofs.len());
            assert_eq!(3, mess.oneofs[0].fields.len());
        }
    }
}
