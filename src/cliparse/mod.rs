use std::mem::MaybeUninit;
use std::sync::{Mutex, Once};

use clap::{Parser, ValueEnum};

///
///  # cargo run -- -h可以打印本程序的 命令行输入指南
///
///  # 参数 & 选项：
/// 	1. 没有arg申明的，都是参数
/// 	2. <>表示必须参数， []表示可选参数。 必须参数必须要在可选参数的前面，按顺序排列
/// 	3. 有arg申明的 ，是选项
/// 	4. long，以全称的方式赋值 --name xxx
/// 	5. short，使用短名,这三个等价 -nxxx -n xxx -n=xxx
/// 	6. bool 类型必须定义成arg. 且输入只能有0或者1个， 表示true或false。否则报错
///
///
/// # 匹配数量
/// 	1. 将变量设为 `Vec<String>` ，即可匹配多个 eg. -n=123123 -n 34
/// 	2. 将变量设为 Option<String> ，即参数是可选的，可以不输入
/// 	3. 否则就是必须要有一个。
///
/// # 匹配多个字符
/// 	1. 不需要定义成arg， 直接定义成name: Vec<String>,
///
/// # 默认值
///		1. #[arg(default_value_t = 2020)]
///
///
/// # 枚举
/// 	1. 定义枚举  #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
/// 	2.  #[arg(value_enum)]
///
/// # 限制值
/// 	1. #[arg(value_parser = clap::value_parser!(u16).range(1..))]
/// 	2. #[arg(value_parser = port_in_range)],定义函数来判断
///
/// # 程序名字，版本：
/// 	1. #[command(name = "MyApp")]
/// 	2. #[command(version = "1.0")]
///
///
///
///
#[command(name = "My Rust Tools")]
#[command(author = "miraclerhe@tencent.com")]
#[command(version = "1.0.0")]
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct MyApp {
    // arguments
    /// 执行本工具的参数
    #[arg(value_enum)]
    pub mode: Mode,

    /// 文件路径数组
    pub path: Vec<String>,

    // options
    /// 是否显示调试信息
    #[arg(long)]
    pub debug: bool,
    /// 如果有目录,就无视前面的路径
    #[arg(short)]
    pub dir: Option<String>,

    /// 输出目录 默认"./"
    #[arg(short)]
    pub output: String,
}

impl IDefault<MyApp> for MyApp {
    fn fill_default(&mut self) {
        if self.output == "" {
            self.output = ".".to_string();
        }
    }

    fn get_default() -> Self {
        MyApp {
            mode: Mode::Proto2lua,
            path: vec![],
            debug: false,
            dir: None,
            output: "".to_string(),
        }
    }

    fn from_other(&mut self, other: MyApp) {
        self.output = other.output;
        self.dir = other.dir;
        self.debug = other.debug;
        self.path = other.path;
        self.mode = other.mode;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    /// Proto2lua 输入要转换的proto文件
    /// # dir usage:
    /// ### cargo run -- proto2lua --debug -d ./src -o ./mod
    /// # files usage:
    /// ### cargo run -- proto2lua --debug ./src/example.proto -o ./mod
    Proto2lua,
    /// 啥也不是,待续
    Nothing,
}

pub trait IDefault<T> {
    fn fill_default(&mut self);
    fn get_default() -> T;
    fn from_other(&mut self, other: T);
}
