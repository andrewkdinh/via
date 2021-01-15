#![allow(dead_code)]
// #![warn(unused_variables)]
// #![warn(unused_mut)]

extern crate termion;
extern crate regex;

use std::env;
use std::fs::File;
use std::io::{BufReader};
// use std::io::prelude::*;
use std::path::Path;
use regex::Regex;

use std::convert::TryInto;

mod editor;
mod piece_table;

// use termion::{color, cursor, clear};
// use termion::event::*;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};

fn main() {
    let args: Vec<String> = env::args().collect();
    // TODO: Match command line options, different order, etc.
    let editor = initialize(process_args(&args));
    let mut piece_table = editor.piece_table;

    // Print the current file text onto screen
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut editor_x: usize = 1;
    let mut editor_y: usize = 0;
    write!(stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)).unwrap();

    for line in piece_table.text().lines() {
        editor_y += 1;
        write!(stdout,
            "{}{}",
            termion::cursor::Goto(editor_x.try_into().unwrap(), editor_y.try_into().unwrap()),
            line).unwrap();
    }
    stdout.flush().unwrap();

    let mut buffer_index = piece_table.text_len();
    let mut cmd_index = 0;
    let mut cmd_piece_table = piece_table::PieceTable::new("".to_string());
    let mut mode = 1; // [0: insert, 1: visual, 2: command]
    for c in stdin.keys() {
        write!(stdout,
               "{}{}",
               termion::cursor::Goto(1, 1),
               termion::clear::CurrentLine)
                .unwrap();

        let i = c.unwrap();
        if i == Key::Esc {
            mode = 1
        } else if mode == 0 {
            // insert mode
            if i == Key::Backspace {
                if buffer_index != 0 {
                    piece_table.delete_text(buffer_index - 1, buffer_index);
                    buffer_index -= 1;

                    if editor_x == 1 {
                        write!(stdout,
                            "{}",
                            termion::cursor::Up(1)).unwrap();
                            // TODO: how to go to right x value?
                    } else {
                        write!(stdout, "{}", termion::cursor::Up(1)).unwrap();
                    }
                }
            } else if i == Key::Up {

            } else if i == Key::Right {
                
            } else if i == Key::Down {

            } else if i == Key::Left {
                
            }
        } else if mode == 1 {
            // visual mode
            if i == Key::Char('i') {
                mode = 0;
            } else if i == Key::Char(':') {
                mode = 2;
                // TODO: Go to bottom line thing
            } else if i == Key::Up || i == Key::Char('k') {
                if let Ok(prev_line) = piece_table.line(editor_y - 2) {
                    let prev_line_len = prev_line.len();
                    buffer_index -= editor_x; // Including the new line
                    if prev_line_len < editor_x - 1 {
                        editor_x = prev_line_len + 1
                    }
                    buffer_index -= prev_line_len - (editor_x - 1);
                    editor_y -= 1;
                    write!(stdout, "{}", termion::cursor::Goto(editor_x.try_into().unwrap(), editor_y.try_into().unwrap())).unwrap()
                }
                // TODO: FIXME
            } else if i == Key::Right || i == Key::Char('l') {
                if let Ok(curr_line) = piece_table.line(editor_y - 1) {
                    if curr_line.len() < editor_x - 1 {
                        editor_x -= 1;
                        write!(stdout, "{}", termion::cursor::Right(1)).unwrap();
                        buffer_index += 1;
                    }
                }
            } else if i == Key::Down || i == Key::Char('j') {
                write!(stdout, "{}", termion::cursor::Down(1)).unwrap();
                // TODO: FIXME
            } else if i == Key::Left || i == Key::Char('h') {
                if editor_y != 1 {
                    write!(stdout, "{}", termion::cursor::Left(1)).unwrap();
                    buffer_index -= 1;
                }
            }
        } else if mode == 2 {
            // command mode
            cmd_piece_table.add_text("q".to_string(), cmd_index);
            cmd_index += 1;
        }
        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

/// Process command line options and return EditorOptions
fn process_args(args: &Vec<String>) -> editor::EditorOptions {
    let mut flags = Vec::new();
    let mut file_names = Vec::new();
    let file_name: String;
    let flags_regex = Regex::new(r"--?\w+").unwrap();
    let default_files = vec!["target/debug/via", "via"];
    for arg in args {
        if flags_regex.is_match(&arg) {
            flags.push(arg);
        } else if !default_files.contains(&arg.as_str()) {
            file_names.push(arg);
        }
    }
    if file_names.len() == 1 {
        file_name = file_names.first().unwrap().to_string();
    } else {
        println!("{:?}", file_names);
        panic!("Must specify a single file to edit"); // Maybe change this to edit multiple files later on
    }
    let mut editor_options = editor::EditorOptions::new(file_name);
    for option in flags {
        if option == "-v" {
            editor_options.verboseness += 1;
        }
    }
    editor_options
}

fn initialize(editor_options: editor::EditorOptions) -> editor::Editor {
    // TODO: Might not want to create file, but instead write to memory then at the end then write to file
    let file_name = &editor_options.file_name;
    let file_path = Path::new(&file_name);
    let file: File;
    if !file_path.is_file() {
        panic!("{} is not a file", file_name);
    } else if file_path.exists() {
        file = File::open(file_name).expect("Failed to open file");
    } else {
        File::create(file_path).expect("Unable to create file");
        file = File::open(file_name).expect("Failed to open file");
    }
    let mut reader = BufReader::new(file);
    // Read until viewport is filled
    // For now, only read 2 lines
    let mut initial_text =String::new();
    let eof_reached = read_lines(&mut reader, 1000000000000000, &mut initial_text); // TODO: FIX ME
    // TODO: Add this to initialization of piece table

    editor::Editor::new(piece_table::PieceTable::new(initial_text), reader, eof_reached, editor_options)
}

/// Read `num_lines` from `reader`, append to str, and returns whether EOF reached
fn read_lines(reader: &mut BufReader<File>, num_lines: usize, str: &mut String) -> bool {
    let mut temp_str = String::new();
    for _ in 0..num_lines {
        // match reader.read_line(&mut temp_str) {
        match std::io::BufRead::read_line(reader, &mut temp_str) {
            Ok(0) => return true,
            Ok(len) => len,
            Err(e) => panic!("Error reading file: {}", e),
        };
        str.push_str(&temp_str);
        temp_str.clear();
    }
    false
}

#[cfg(test)]
mod tests {
    /*
    use super::*;

    #[test]
    fn init() {
        let args: Vec<String> = vec!["unittests/1.in".to_string()];
        let editor = initialize(process_args(&args));
    }
    */
}