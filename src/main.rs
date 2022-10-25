#[macro_use]
extern crate nom;

use std::mem::MaybeUninit;
use std::sync::Once;

use clap::Parser;

use cliparse::Mode;
use cliparse::MyApp;

use crate::cliparse::IDefault;

mod cliparse;
mod proto_2_lua;
mod proto2lua;

fn main() {
    let mut my_app: MyApp = MyApp::parse();
    my_app.fill_default();

    if my_app.debug {
        eprintln!("my_app = {:#?}", my_app);
    }

    match my_app.mode {
        Mode::Proto2lua => {
            proto_2_lua::main(my_app);
        }
        Mode::Nothing => {
            println!("to be continued");
        }
        Mode::Proto2luaNew => {
            proto2lua::main(my_app);
        }
    }

    println!("success");
}
fn init_config(default: Option<MyApp>) -> &'static MyApp {
    // 使用MaybeUninit延迟初始化
    static mut CONF: MaybeUninit<MyApp> = MaybeUninit::uninit();
    // Once带锁保证只进行一次初始化
    static ONCE: Once = Once::new();

    match default {
        None => {}
        Some(v) => {
            ONCE.call_once(|| unsafe {
                CONF.as_mut_ptr().write(MyApp { ..v });
            });
        }
    }

    unsafe { &(*CONF.as_ptr()) }
}

fn get_config() -> &'static MyApp {
    init_config(None)
}
