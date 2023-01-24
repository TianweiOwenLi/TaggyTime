
mod tasks_util;
mod time;
mod taggy_io;

use tasks_util::Tasks;
use time::MinInstant;

use std::mem::size_of;

use crate::time::Date;

fn main() {
    println!("Hello, world!");
    println!("{}", size_of::<Tasks>());
    let x = MinInstant::now(0);
    println!("{}", Date::from_min_instant(x));
}
