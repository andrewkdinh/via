#![allow(dead_code)]
// #![warn(unused_variables)]
// #![warn(unused_mut)]

use std::env;

mod modules;

use modules::via::Via;

fn main() {
    let (mut file_paths, options) = Via::process_args(env::args().collect());
    if file_paths.is_empty() {
        file_paths.push(String::new());
    }
    for file_path in file_paths {
        let mut via = Via::new(file_path, options.clone());
        via.init();
    }
}
