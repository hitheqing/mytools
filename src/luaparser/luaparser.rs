use std::path::{Path, PathBuf};
use std::{ffi, ptr, str};
use std::str::FromStr;
use libc;
use nom::{digit, hex_digit, IResult, multispace, space};
use nom::IResult::Error;
use walkdir::WalkDir;
pub type LuaFloat = f64;
named!(
    lua_block_comment<()>,
    do_parse!(
        tag!("--[[")
        >> take_until_and_consume!("]]")
        >>()
    )
);

named!(
    lua_line_comment<String>,
    do_parse!(
        tag!("--")
        >> c:take_until_and_consume!("\n")
        >>(String::from_utf8(c.to_vec()).unwrap_or_default())
    )
);

named!(
    br<()>,
    alt!(lua_block_comment|map!(multispace, |_| ()))
);

fn is_word(b: u8) -> bool {
    match b {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' => true,
        _ => false,
    }
}

named!(word<String>, map_res!(take_while!(is_word), |b: &[u8]| String::from_utf8(b.to_vec())));

// local a
named!(
    local_define<String>,
    do_parse!(tag!("local")
            >> many1!(br)
            >> w:word
            >>(w)
        )
);

// local a = xx
named!(
    local_assign<String>,
    do_parse!(tag!("local")
        >> many1!(br)
        >> w:word
        >> many0!(br)
        >> tag!("=")
        >> many0!(br)
        >> lv:lua_value
            >>(w)
    )
);
named!(
    lua_string_1<String>,
    do_parse!(
        tag!("'")
        >> c:take_until_and_consume!("'")
        >> c:is_not!("'")
        >>(String::from_utf8(c.to_vec()).unwrap_or_default())
    )
);
named!(
    lua_string_2<String>,
    do_parse!(
        tag!("\"")
        >> c:take_until_and_consume!("\"")
        >>(String::from_utf8(c.to_vec()).unwrap_or_default())
    )
);


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Numeral {
    Float(LuaFloat),
    Int(isize)
}

impl Numeral {
    fn from_full_dec(digits: &[u8]) -> Numeral {
        let s = str::from_utf8(digits).unwrap();
        if let Ok(n) = isize::from_str_radix(s, 10) {
            Numeral::Int(n)
        }
        else {
            let f = LuaFloat::from_str(s).unwrap();
            Numeral::Float(f)
        }
    }

    fn from_full_hex(digits: &[u8]) -> Numeral {
        let s = str::from_utf8(digits).unwrap();
        // Slice off the '0x' from the string and try parsing as a normal integer
        if let Ok(n) = isize::from_str_radix(&s[2..], 16) {
            Numeral::Int(n)
        }
        // Otherwise, parse as a float
        else {
            let cstr = ffi::CString::new(digits).unwrap();
            let f = unsafe { libc::strtod(cstr.into_raw() as *const _, ptr::null_mut()) };
            Numeral::Float(f as LuaFloat)
        }
    }
}

named!(pub decimal<usize>,
   map_res!(
       map_res!(
           call!(digit),
           str::from_utf8),
       |s| usize::from_str_radix(s, 10)
   )
);

named!(pub hex<usize>,
   map_res!(
       map_res!(
           call!(hex_digit),
           str::from_utf8),
       |s| usize::from_str_radix(s, 16)
   )
);

named!(float_sgn_suffix<i32>,
   map!(
       do_parse!(
           sign: opt!(alt!(tag!("+") | tag!("-"))) >>
           expt: decimal >>
           (sign, expt)
       ),
       |(sign, expt): (Option<&[u8]>, usize)| {
           match sign {
               Some(b"+") | None => expt as i32,
               Some(b"-") => -(expt as i32),
               _ => unreachable!(),
           }
       }
    )
);

named!(float_mag<i32>, preceded!(alt!(tag!("e") | tag!("E")), float_sgn_suffix));
named!(float_pow<i32>, preceded!(alt!(tag!("p") | tag!("P")), float_sgn_suffix));

named!(hex_lit<Numeral>,
    map!(
        recognize!(
            tuple!(
               opt!(tag!("-")),
               alt!(tag!("0x") | tag!("0X")),
               hex_digit,
               opt!(complete!(preceded!(tag!("."), hex_digit))),
               opt!(complete!(float_pow))
            )
        ),
        Numeral::from_full_hex
    )
);

named!(dec_lit<Numeral>,
    map!(
        recognize!(
            tuple!(
               opt!(tag!("-")),
               digit,
               opt!(complete!(preceded!(tag!("."), digit))),
               opt!(complete!(float_mag))
            )
        ),
        Numeral::from_full_dec
    )
);

named!(pub num_lit<Numeral>, alt!(hex_lit | dec_lit));


named!(
    lua_string<String>,
    alt!(
        lua_string_1 |lua_string_2
    )
);

named!(
    lua_nil<()>,
    do_parse!(
        tag!("nil")>>()
    )
);

named!(
    lua_value<LuaValue>,
    alt!(
        lua_string => { |f| LuaValue::String(f) }
        | num_lit => { |f| LuaValue::Number(f) }
        | lua_nil => { |f| LuaValue::Nil }
    )
);

enum  LuaValue{
    String(String),
    Number(Numeral),
    Nil,
    Table,
    Function,
}

#[test]
fn t_number (){
    let good_inputs = ["13", "0xf5", "0x10abcp-7", "0X302498a.e10", "0.00271828e3"];
    let bad_inputs = ["f5", "13f5", "10.3.1.4", "0xp1", "50e"];
    let good_outputs = vec![
        Numeral::Int(13),
        Numeral::Int(0xf5),
        Numeral::Float(533.46875),
        Numeral::Float(50481546.87890625),
        Numeral::Float(2.71828),
    ];
    // We won't be matching the entirety of the input
    let bad_outputs = vec![
        IResult::Error(nom::ErrorKind::Alt),
        IResult::Done(&b"f5"[..], Numeral::Int(13)),
        IResult::Done(&b".1.4"[..], Numeral::Float(10.3)),
        IResult::Done(&b"xp1"[..], Numeral::Int(0)),
        IResult::Done(&b"e"[..], Numeral::Int(50)),
    ];

    for (input, expected) in good_inputs.iter().zip(good_outputs.into_iter()) {
        println!("trying {:?}", input);
        assert_eq!(num_lit(input.as_bytes()), IResult::Done(&b""[..], expected));
    }

    for (input, expected) in bad_inputs.iter().zip(bad_outputs.into_iter()) {
        assert_eq!(num_lit(input.as_bytes()), expected);
    }
}

#[test]
fn t_st(){
    let s = r#""asfdg\"dsd""#;
    let ss = lua_string(s.as_bytes());
    if ss.is_done() {
        eprintln!("ss.unwrap() = {:#?}", ss.unwrap().1);
        // assert!(true,"{}",ss.unwrap().1)
    }else {
        assert!(false);
    }
}