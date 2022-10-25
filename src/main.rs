#[macro_use]
extern crate nom;

use clap::{Parser, ValueEnum};
use clap::{Arg, ArgAction, command};

use cliparse::Mode;
use cliparse::MyApp;

use crate::cliparse::IDefault;

mod cliparse;
mod proto_2_lua;
mod proto2lua;

fn main() {
    // let matches = command!() // requires `cargo` feature
    //     .arg(
    //         Arg::new("verbose")
    //             .short('s')
    //             .long("verbose")
    //             .action(ArgAction::Count),
    //     )
    //     .get_matches();
    //
    // println!("verbose: {:?}", matches.get_count("verbose"));


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
    }

    println!("success");
}
