
mod tasks_util;
mod time;

use tasks_util::tasks;
use time::MinInstant;

use std::mem::size_of;

use crate::time::Date;

fn main() {
    println!("Hello, world!");
    println!("{}", size_of::<tasks>());
    let x = MinInstant::now(0);
    println!("{}", Date::from_min_instant(x));
}
