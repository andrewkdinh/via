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

mod editor;
mod piece_table;

// use termion::{color, cursor, clear};
// use termion::event::*;
use termion::async_stdin;
use termion::input::{MouseTerminal};
use termion::raw::IntoRawMode;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    // TODO: Match command line options, different order, etc.
    let editor_options = process_args(&args);
    /*
    let file_name = match args.last() {
        Some(file_name) => file_name,
        None => panic!("Please specify a file to edit"),
    };
    */
    let editor = initialize(editor_options);
    let mut piece_table = editor.piece_table;

    // Print the current file text onto screen
    let stdin = async_stdin();
    let mut stdout = MouseTerminal::from(io::stdout().into_raw_mode().unwrap());
    write!(stdout, "{}{}{}{}", 
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        piece_table.text(),
        termion::cursor::Hide).unwrap();
    stdout.flush().unwrap();

    /*
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Esc                        => normal_mode(),
            Key::Char(':')                  => colon_mode(),
            Key::Char('i')                  => insert_mode(c.unwrap()),
            Key::Char('v') | Key::Char('V') => visual_mode(c.unwrap()),
            Key::Left | Key::Right          => movement_horizontal(c.unwrap()),
            Key::Up | Key::Down             => movement_vertical(c.unwrap()),
            Key::Char(c)                    => add_text_buffered(c),
            _                               => unsupported(),
        }
        stdout.flush().unwrap();
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();
    */
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
        panic!("Must specify only one filed to edit"); // Maybe change this to edit multiple files later on
    }
    let mut editor_options = editor::EditorOptions::new(file_name);
    for option in flags {
        /*
        Example:
        if (option == "-v") {
            editor_options.verbose = true;
        }
        */
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
    let eof_reached = read_lines(&mut reader, 2, &mut initial_text);
    let text_len = initial_text.len();
    let mut original_buffer = piece_table::Buffer::new();
    original_buffer.update_add(initial_text);
    let mut piece_table = piece_table::PieceTable::new(original_buffer);
    let mut first_entry = piece_table::TableEntry::new(false, 0, text_len);
    if text_len == 0 {
        first_entry.active = false;
    }
    piece_table.push(first_entry);

    editor::Editor::new(piece_table, reader, eof_reached, editor_options)
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