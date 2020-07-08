#![allow(dead_code)]
#![warn(unused_variables)]
// #![warn(unused_mut)]

use std::fs::File;
// use std::io;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
/// The overarching editor class
struct Editor {
    piece_table: PieceTable,
    reader: BufReader<File>,
}

impl Editor {
    /// Initialize a new editor with a piece table and reader
    fn new(piece_table: PieceTable, reader: BufReader<File>) -> Editor {
        Editor {piece_table: piece_table, reader: reader}
    }
}

#[derive(Debug)]
/// The main structure for storing text
struct PieceTable {
    table: Vec<TableEntry>,
    original_buffer: Buffer,
    add_buffer: Buffer,
}

impl PieceTable {
    /// Initializes a piece table with the original buffer set
    fn new(original_buffer: Buffer) -> PieceTable {
        PieceTable {table: Vec::new(), original_buffer: original_buffer, add_buffer: Buffer::new()}
    }

    /// Add text at a certain index
    fn add_text(&mut self, text: String, cursor: usize) {
        let text_len = self.text().len();
        if cursor == 0 {
            self.table.insert(0, TableEntry::new(true, 0, text.len()));
        } else if cursor == text_len { // TODO: Decide if I should support -1 or not
            let add_buffer_len = self.add_buffer.text().len();
            self.table.push(TableEntry::new(true, add_buffer_len, add_buffer_len + text.len()));
        } else if cursor > text_len {
            panic!("cursor ({}) is a greater value than text len ({})", cursor, text_len);
        } else {
            let mut table_entry_count = 0;
            let mut curr_pos = 0;
            for table_entry in &mut self.table {
                if table_entry.active {
                    let len = table_entry.end_index - table_entry.start_index;
                    if curr_pos + len > cursor {
                        // TODO: Split the table entry into two
                        table_entry_count += 1;
                    } else if curr_pos == cursor {
                        break
                    } else {
                        curr_pos += len;
                    }
                }
                table_entry_count += 1;
            }
            let add_buffer_len = self.add_buffer.text().len();
            self.table.insert(table_entry_count, TableEntry::new(true, add_buffer_len, add_buffer_len + text.len()));
        }
        self.add_buffer.add_and_update(text);
    }

    /// Returns the text represented by a table entry.
    /// 
    /// Assumes buffer is up to date
    fn table_entry_text(&self, table_entry: &TableEntry) -> &str {
        let buffer = if table_entry.is_add_buffer {&self.add_buffer} else {&self.original_buffer};
        assert_eq!(buffer.text_up_to_date, true);
        match buffer.text().get(table_entry.start_index..table_entry.end_index) {
            Some(text) => text,
            None => panic!("Unable to get {}[{}..{}]", buffer.text(), table_entry.start_index, table_entry.end_index),
        }
    }

    /// Returns all visible text.
    fn text(&mut self) -> String {
        self.original_buffer.update_text();
        self.add_buffer.update_text();

        let mut text = String::new();
        for table_entry in &self.table {
            if table_entry.active {
                text.push_str(self.table_entry_text(table_entry));
            }
        }
        text
        
    }

    /// Insert a table entry to a specific index
    fn insert(&mut self, piece: TableEntry, index: usize) {
        self.table.insert(index, piece);
    }

    /// Add a table entry to the end of the table
    fn push(&mut self, piece: TableEntry) {
        self.table.push(piece);
    }
}

#[derive(Debug)]
/// An entry in PieceTable's table
struct TableEntry {
    is_add_buffer: bool,
    start_index: usize,
    end_index: usize,
    active: bool,
}

impl TableEntry {
    /// Initalize a table entry
    fn new(is_add_buffer: bool, start_index: usize, end_index: usize) -> TableEntry {
        TableEntry {is_add_buffer: is_add_buffer, start_index: start_index, end_index: end_index, active: true}
    }
}

#[derive(Debug)]
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

    fn text(&self) -> &str {
        &self.text
    }

    /// Make `self.text` up to date.
    /// 
    /// Returns whether text was updated or not
    fn update_text(&mut self) -> bool {
        if self.text_up_to_date {
            return false;
        }
        // TODO: Change this to keep track of self.text_pieces_index to be more efficient
        // e.g. start at the last added index rather than creating from scratch every time
        self.text.clear();
        for s in &self.text_pieces {
            self.text.push_str(&s);
        }
        self.text_up_to_date = true;
        true
    }

    /// Add a text piece to text pieces
    fn add(&mut self, text: String) {
        self.text_pieces.push(text);
        self.text_up_to_date = false;
    }

    /// Add a text piece AND update `self.text`
    fn add_and_update(&mut self, text: String) {
        self.text.push_str(text.as_str());
        self.text_pieces.push(text);
    }
}

fn main() {
    let file_name = String::from("test.txt"); // TODO: Change this to be the input
    let editor = initialize(&file_name);
    let mut piece_table = editor.piece_table;

    // TODO: Add a table entry depending on where the text is added
    // println!("{}", piece_table.add_buffer.text());
    // piece_table.push(TableEntry::new(true, 0, add_text_len));
    piece_table.add_text(String::from("a"), 0);
    piece_table.add_text(String::from("e"), 3);
    // piece_table.add_text(String::from("c"), 2);
    println!("{}", piece_table.text());
}

fn initialize(file_name: &String) -> Editor {
    let f = File::open(file_name).expect("Failed to open file");
    let mut reader = BufReader::new(f);
    // Read until viewport is reached
    // For now, only read 2 lines
    let initial_text = read_lines(&mut reader, 2);
    let text_len = initial_text.len();
    let mut original_buffer = Buffer::new();
    original_buffer.add_and_update(initial_text);
    let mut piece_table = PieceTable::new(original_buffer);
    piece_table.push(TableEntry::new(false, 0, text_len));

    Editor::new(piece_table, reader)
}

fn read_lines(reader: &mut BufReader<File>, num_lines: u8) -> String {
    let mut final_str = String::new();
    let mut temp_str = String::new();
    for _ in 0..num_lines {
        match reader.read_line(&mut temp_str) {
            Ok(0) => break, // TODO: Handle EOF better
            Ok(len) => len,
            Err(e) => panic!("Error reading file: {:?}", e),
        };
        final_str.push_str(&temp_str);
        temp_str.clear();
    }
    final_str
}