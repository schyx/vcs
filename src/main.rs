use std::env;

use operations::{
    add::add, branch::branch, checkout::checkout, commit::commit, init::init, log::log,
    merge::merge, rm::rm, status::status,
};

pub mod objects;
pub mod operations;
pub mod utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => {
            print!("{}", init(&args));
        }
        "add" => {
            let (text, _) = add(&args).unwrap();
            print!("{}", text);
        }
        "rm" => {
            let text = rm(&args).unwrap();
            print!("{}", text);
        }
        "commit" => {
            let (text, _) = commit(&args).unwrap();
            print!("{}", text);
        }
        "branch" => {
            let text = branch(&args).unwrap();
            print!("{}", text);
        }
        "checkout" => {
            let text = checkout(&args).unwrap();
            print!("{}", text);
        }
        "log" => {
            let text = log(&args).unwrap();
            print!("{}", text);
        }
        "merge" => {
            let text = merge(&args).unwrap();
            print!("{}", text);
        }
        "status" => {
            let text = status(&args).unwrap();
            print!("{}", text);
        }
        _ => {
            println!("No command with that name exists")
        }
    }
}
