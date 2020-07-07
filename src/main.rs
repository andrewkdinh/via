#![allow(dead_code)]
// #![warn(unused_mut)]
// #![warn(unused_variables)]

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::convert::TryInto;

/// The main structure for storing text
struct PieceTable {
    table: Vec<TableEntry>,
    original_buffer: Buffer,
    add_buffer: Buffer,
}

impl PieceTable {
    /// Initializes a piece table with only the original buffer set
    fn new(original_buffer: Buffer) -> PieceTable {
        PieceTable {table: Vec::new(), original_buffer: original_buffer, add_buffer: Buffer::new()}
    }
    /*
    /// Initializes a piece table with all traits empty
    fn new() -> PieceTable {
        PieceTable {table: Vec::new(), original_buffer: Buffer::new(), add_buffer: Buffer::new()}
    }
    */

    /// Add a table entry to the end of the table
    fn append(&mut self, piece: TableEntry) {
        self.table.push(piece);
    }
}

/// An entry in PieceTable's table
struct TableEntry {
    is_add_buffer: bool,
    start_index: u64,
    end_index: u64,
}

impl TableEntry {
    /// Initalize a table entry
    fn new(is_add_buffer: bool, start_index: u64, end_index: u64) -> TableEntry {
        TableEntry {is_add_buffer: is_add_buffer, start_index: start_index, end_index: end_index}
    }
}

/// Immutable text (abstracted)
struct Buffer {
    text: String,
    text_up_to_date: bool,
    text_pieces: Vec<String>,
}

impl Buffer {
    /// Initializes the buffer with an empty string
    fn new() -> Buffer {
        Buffer {text: String::from(""), text_up_to_date: true, text_pieces: Vec::new()}
    }

    /// Add a text piece to text pieces
    fn add(&mut self, text: String) {
        self.text_pieces.push(text)
    }
}

fn main() {
    /*
    let file_name = String::from("test.txt");
    let contents = match read_from_file(file_name) {
        Ok(contents) => contents,
        Err(e) => panic!("Error reading file: {:?}", e),
    };
    */
    let file_name = String::from("../Cargo.toml"); // TODO: Change this to be the input
    let f = File::open(file_name).expect("Failed to open file");
    let reader = BufReader::new(f);
    // Read until viewport is reached
    // For now, only read 2 lines
    let text = read_lines(reader, 2);
    let text_len = text.len().try_into().unwrap();
    let mut piece_table = PieceTable::new(Buffer {text: text, text_up_to_date: true, text_pieces: Vec::new()});
    piece_table.append(TableEntry::new(true, 0, text_len));

    // print!("{}", piece_table.original_buffer.text);
    // TODO: For any text added, add to _addBuffer
}

fn read_lines(mut reader: BufReader<File>, num_lines: u8) -> String {
    let mut final_str = String::new();
    let mut temp_str = String::new();
    for _ in 0..num_lines {
        match reader.read_line(&mut temp_str) {
            Ok(0) => break, // TODO: Handle EOF better
            Ok(len) => len,
            Err(e) => panic!("Error reading file: {:?}", e),
        };
        final_str.push_str(&temp_str);
        temp_str = String::new();
    }
    final_str
}