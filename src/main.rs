use std::path::Path;

mod lib;
mod proto_2_lua;

static mut COMMAND_LINE_OP: CommandLineOp = CommandLineOp { show_func_write: false };

// 第一个程序：读文件、正则匹配、文件结构parse、写文件

fn main() {
    for x in std::env::args() {
        println!("---arg:{}", x);
        if x == "proto_2_lua" {
            proto_2_lua::main();
            return;
        }
    }
}

struct CommandLineOp {
    show_func_write: bool,
}
