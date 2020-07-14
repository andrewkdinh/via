// mod piece_table;

use std::fs::File;
use std::io::{BufReader};

#[derive(Debug)]
/// The overarching editor class
pub(crate) struct Editor {
    /// The piece table
    pub(crate) piece_table: super::piece_table::PieceTable,
    /// Reader of the file
    reader: BufReader<File>,
    /// Whether we have read all of `self.reader`
    eof_reached: bool,
    /// Configured options
    options: EditorOptions,
}

impl Editor {
    /// Initialize a new editor with a piece table and reader
    pub(crate) fn new(piece_table: super::piece_table::PieceTable, reader: BufReader<File>, eof_reached: bool, options: EditorOptions) -> Editor {
        Editor {piece_table: piece_table, reader: reader, eof_reached: eof_reached, options: options}
    }

    /// Read to end of the file and add it to `piece_table.original_buffer`
    fn read_to_eof(&mut self) {
        let mut final_str = String::new();
        let mut temp_str = String::new();
        loop {
            // match self.reader.read_line(&mut temp_str) {
            match std::io::BufRead::read_line(&mut self.reader, &mut temp_str) {
                Ok(0) => break,
                Ok(len) => len,
                Err(e) => panic!("Error reading file: {:?}", e),
            };
            final_str.push_str(&temp_str);
            temp_str.clear();
        }
        self.piece_table.original_buffer.update_add(final_str);
    }
}

#[derive(Debug)]
pub(crate) struct EditorOptions {
    pub(crate) file_name: String,
}

impl EditorOptions {
    /// Return default options
    pub(crate) fn new(file_name: String) -> EditorOptions {
        EditorOptions {file_name: file_name}
    }
}