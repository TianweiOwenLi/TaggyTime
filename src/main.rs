
mod tasks_util;
mod time;
mod command_parser;

mod todo;

mod percent;

mod ics_parser;

use tasks_util::Tasks;
use time::MinInstant;


use crate::time::Date;
// use crate::taggy_io::datapoint;



/// handles user-input command.
fn handle_user_command<'a>(cmd: &String) -> Result<(), &'a str> { // TODO fg/bg
  if cmd.starts_with("log ") {
    println!("[taggytime] logged msg {:?}", Tasks::from_str(cmd[4..].to_string()).unwrap());
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
  
  // cli mode
  match ag.get(1) {
    Some(s) => // cli mode
    match handle_user_command(s) {
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

        if let Err(read_err) = x.read_line(&mut buf) {
          print!("[taggytime] Failed to read line: {read_err}");
        } else if let Err(emsg) = 
          handle_user_command(&buf) {
          println!("[taggytime] Error: {}", emsg)
        }
      }
    }
  }   

}
