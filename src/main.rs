#![allow(dead_code)]
#![warn(unused_variables)]
// #![warn(unused_mut)]

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
/// The overarching editor class
struct Editor {
    /// The piece table
    piece_table: PieceTable,
    /// Reader of the file
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
    /// The main table, contains `TableEntry`'s
    table: Vec<TableEntry>,
    /// Original buffer
    original_buffer: Buffer,
    /// Add buffer
    add_buffer: Buffer,
    /// All active text. Only to be used when `text_up_to_date == true`
    text: String,
    /// Whether `text` is up to date
    text_up_to_date: bool,
    /// List of table entries that have had actions.
    actions: Vec<TableEntry>,
    /// Where in `self.actions` we are currently at
    actions_index: usize,
}

impl PieceTable {
    /// Initializes a piece table with the original buffer set
    fn new(original_buffer: Buffer) -> PieceTable {
        PieceTable {table: Vec::new(), original_buffer: original_buffer, add_buffer: Buffer::new(), text: String::new(), text_up_to_date: true, actions: Vec::new(), actions_index: 0}
    }

    /// Add text at a certain index
    fn add_text(&mut self, text: String, cursor: usize) {
        let text_len = self.text_len();
        let add_buffer_len = self.add_buffer.text().len();
        if cursor > text_len {
            panic!("cursor ({}) is a greater value than text len ({})", cursor, text_len);
        } else if cursor == 0 {
            let new_table_entry = TableEntry::new(true, add_buffer_len, add_buffer_len + text.len());
            self.add_action(&new_table_entry);
            self.table.insert(0, new_table_entry);
        } else if cursor == text_len {
            let new_table_entry = TableEntry::new(true, add_buffer_len, add_buffer_len + text.len());
            self.add_action(&new_table_entry);
            self.table.push(new_table_entry);
        } else {
            let mut table_entry_count = 0;
            let mut curr_pos = 0;
            for table_entry in &mut self.table {
                if table_entry.active {
                    let len = table_entry.end_index - table_entry.start_index;
                    if curr_pos + len > cursor {
                        let last_table_entry_length = len - (cursor - curr_pos);
                        let old_table_entry_end_index = table_entry.end_index;

                        table_entry.end_index = table_entry.end_index - last_table_entry_length;
                        let last_table_entry = TableEntry::new(table_entry.is_add_buffer, table_entry.end_index, old_table_entry_end_index);
                        self.table.insert(table_entry_count + 1, last_table_entry);

                        let middle_table_entry = TableEntry::new(true, add_buffer_len, add_buffer_len + text.len());
                        self.add_action(&middle_table_entry);
                        self.table.insert(table_entry_count + 1, middle_table_entry);
                        break
                    } else if curr_pos == cursor {
                        let new_table_entry = TableEntry::new(true, add_buffer_len, add_buffer_len + text.len());
                        self.add_action(&new_table_entry);
                        self.table.insert(table_entry_count, new_table_entry);
                        break
                    }
                    curr_pos += len;
                }
                table_entry_count += 1;
            }
        }
        self.add_buffer.update_add(text);
        self.text_up_to_date = false;
    }

    /// Delete text from `start` to `end`
    fn delete_text(&mut self, start: usize, end: usize) {
        let text_len = self.text_len();
        if start >= end || end == 0 || end > text_len  {
            panic!("Can't delete from start ({}) to end ({}) of text size {}", start, end, self.text_len());
        }
        let mut curr_pos = 0;
        let mut table_entry_count = 0;

        let mut temp_table_entry = TableEntry::new(true, 0, 0);
        let mut temp_table_set = false;
        let mut temp_table_index = 0;
        for table_entry in self.table.iter_mut() {
            if curr_pos == end {
                break
            }
            if table_entry.active {
                let len = table_entry.end_index - table_entry.start_index;
                if curr_pos <= start && curr_pos + len > start && curr_pos + len <= end {
                    // At table entry to start at and split, but possibly continue
                    let split_point = start - curr_pos;
                    temp_table_entry = TableEntry::new(table_entry.is_add_buffer, table_entry.start_index, table_entry.start_index + split_point);
                    table_entry.start_index = table_entry.start_index + split_point;
                    table_entry.active = false;

                    temp_table_index = table_entry_count;
                    temp_table_set = true;
                } else if curr_pos >= start && curr_pos + len <= end {
                        // Start is this/before this cell & end is this/after this cell
                        table_entry.active = false;
                } else if curr_pos >= start && curr_pos + len > end {
                    // At the table entry to end at and split
                    let split_point = end - curr_pos;
                    let mut temp_table_entry = TableEntry::new(table_entry.is_add_buffer, table_entry.start_index, table_entry.start_index + split_point);
                    temp_table_entry.active = false;
                    table_entry.start_index = table_entry.start_index + split_point;
                    self.table.insert(table_entry_count, temp_table_entry);
                    break
                }
                curr_pos += len;
            }
            table_entry_count += 1;
        }
        if temp_table_set {
            self.table.insert(temp_table_index, temp_table_entry);
        }
        self.text_up_to_date = false;
    }

    /// Add a table entry to actions
    fn add_action(&mut self, table_entry: &TableEntry) {
        // Remove actions after current index
        self.actions = self.actions[..self.actions_index].to_vec();
        self.actions.push(*table_entry);
        self.actions_index += 1;
    }

    /// Undo an action. Errors if no actions to undo
    fn undo(&mut self) {
        // TODO: Need to support undoing delete's, which can span multiple table entries
        if self.actions.is_empty() {
            panic!("Unable to undo");
        }
        let table_entry_copy : &TableEntry = match self.actions.get(self.actions_index - 1) {
            Some(table_entry) => table_entry,
            None => panic!("Unable to get last action"),
        };
        for table_entry in self.table.iter_mut() {
            if table_entry.is_add_buffer == table_entry_copy.is_add_buffer
                && table_entry.start_index == table_entry_copy.start_index
                && table_entry.end_index == table_entry_copy.end_index {
                    table_entry.switch();
                    break
                }
        }
        self.text_up_to_date = false;
        self.actions_index -= 1;
    }

    /// Redo an action. Errors if no actions to redo
    fn redo(&mut self) {
        // TODO
        self.text_up_to_date = false;
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
    fn text(&mut self) -> &str {
        let a = self.original_buffer.update_text();
        let b = self.add_buffer.update_text();
        if self.text_up_to_date && !a && !b {
            return &self.text;
        }

        // TODO: Be more efficient about doing this
        let mut text = String::new();
        for table_entry in &self.table {
            if table_entry.active {
                text.push_str(self.table_entry_text(table_entry));
            }
        }
        self.text = text;
        self.text_up_to_date = true;
        &self.text
    }

    /// A slightly more efficient way of calculating the length of `self.text().len()`
    fn text_len(&self) -> usize {
        if self.text_up_to_date {
            return self.text.len();
        }

        // TODO: Be more efficient about doing this
        let mut len = 0;
        for table_entry in &self.table {
            if table_entry.active {
                len += table_entry.end_index - table_entry.start_index;
            }
        }
        len
    }

    /// Insert a table entry to a specific index
    fn insert(&mut self, piece: TableEntry, index: usize) {
        self.table.insert(index, piece);
    }

    /// Add a table entry to the end of the table
    fn push(&mut self, piece: TableEntry) {
        self.table.push(piece);
        self.text_up_to_date = false;
    }
}

#[derive(Copy, Clone)] // Needed for PieceTable.actions
#[derive(Debug)]
/// An entry in PieceTable's table
struct TableEntry {
    /// Whether this table entry points to the add buffer
    is_add_buffer: bool,
    /// Start index
    start_index: usize,
    /// End index
    end_index: usize,
    /// Whether this table is visible
    active: bool,
}

impl TableEntry {
    /// Initalize a table entry
    fn new(is_add_buffer: bool, start_index: usize, end_index: usize) -> TableEntry {
        TableEntry {is_add_buffer: is_add_buffer, start_index: start_index, end_index: end_index, active: true}
    }

    /// Change from active to deactivated and visa versa
    fn switch(&mut self) {
        self.active = !self.active;
    }
}

#[derive(Debug)]
/// Immutable text (abstracted)
struct Buffer {
    /// Text contained in this buffer. Only use when `text_up_to_date == true`
    text: String,
    /// Whether `text` is up to date
    text_up_to_date: bool,
    /// Text pieces making up `text`
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
    fn update_add(&mut self, text: String) {
        if self.text_up_to_date {
            self.text.push_str(text.as_str());
            self.text_pieces.push(text);
        } else {
            self.add(text);
            self.update_text();
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // TODO: Match command line options, different order, etc.
    let file_name = match args.last() {
        Some(file_name) => file_name,
        None => panic!("Please specify a file to edit"),
    };
    let editor = initialize(&file_name);
    let mut piece_table = editor.piece_table;

    println!("Org: {}", piece_table.text());
    piece_table.add_text(String::from("ac"), 0);
    piece_table.add_text(String::from("b"), 1);
    piece_table.add_text(String::from("de"), 3);
    println!("New: {}", piece_table.text());
    /*
    piece_table.delete_text(0, 1);
    println!("New: {}", piece_table.text());
    piece_table.delete_text(4, 5);
    println!("New: {}", piece_table.text());
    piece_table.delete_text(4, 5);
    println!("New: {}", piece_table.text());
    */
    piece_table.undo();
    piece_table.undo();
    println!("New: {}", piece_table.text());
}

fn initialize(file_name: &String) -> Editor {
    let f = File::open(file_name).expect("Failed to open file");
    let mut reader = BufReader::new(f);
    // Read until viewport is reached
    // For now, only read 2 lines
    let initial_text = read_lines(&mut reader, 2);
    let text_len = initial_text.len();
    let mut original_buffer = Buffer::new();
    original_buffer.update_add(initial_text);
    let mut piece_table = PieceTable::new(original_buffer);
    let mut first_entry = TableEntry::new(false, 0, text_len);
    if text_len == 0 {
        first_entry.active = false;
    }
    piece_table.push(first_entry);

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