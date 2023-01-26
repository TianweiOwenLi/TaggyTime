
mod tasks_util;
mod time;
mod taggy_io;
mod command_parser;

use tasks_util::Tasks;
use time::MinInstant;

use std::mem::size_of;

use crate::time::Date;
use crate::taggy_io::{record};
// use crate::taggy_io::datapoint;

/// handles user-input command.
fn handle_user_command<'a>(rc: &'a mut record, cmd: &String) -> Result<(), &'a str> { // TODO fg/bg
    if cmd.starts_with("log ") {
        println!("[taggytime] logged msg {}", &cmd[4..]);
        Ok(())
    } else if cmd.starts_with("top") {
        println!("[taggytime] accessed top");
        Ok(())
    } else if cmd.starts_with("add-todo") {
        println!("[taggytime] added todo with msg {}", &cmd[8..]);
        Ok(())
    } else if cmd.starts_with("remove-todo") { // TODO fuzzy remove
        println!("[taggytime] removed todo with msg {}", &cmd[11..]);
        Ok(())
    } else {
        Err("Command not found")
    }
}

fn main() {

    let ag: Vec<String> = std::env::args().collect();
    
    // initialize
    let mut rc: record = record::empty();


    // cli mode
    match ag.get(1) {
        Some(s) => // cli mode
        match handle_user_command(&mut rc, s) {
            Ok(_) => {
                println!("[taggytime] handled");
                std::process::exit(0)
            },
            Err(emsg) => {
                println!("[taggytime] Error: {}", emsg);
                std::process::exit(1)
            }
        },

        None => // interactive mode
        {
            let x = std::io::stdin();
            loop { // TODO prettify input ui

                let mut buf = String::new();
                match x.read_line(&mut buf) {
                    Ok(_) => {
                        match handle_user_command(&mut rc, &buf) {
                            Ok(_) => println!("[taggytime] handled"),
                            Err(emsg) => println!("[taggytime] Error: {}", emsg),
                        };
                        continue;
                    },
                    Err(read_err) => {
                        print!("[taggytime] Failed to read line: {read_err}");
                        continue;
                    },
                }
            }
        }
    }

    // interactive mode
    
    

}
