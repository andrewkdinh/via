use std::fs::File;
use std::io::{BufRead, BufReader};
// use std::env;
use std::cmp::min;
use std::path::Path;

use super::piece_table::PieceTable;

#[derive(Debug)]
/// An editor window
pub(crate) struct Editor {
    /// The piece table
    piece_table: PieceTable,
    /// Index we are currently at in `self.piece_table` (0-indexed)
    pt_index: usize,
    /// Path of file being editing (may not yet exist). 
    /// Empty string if no file specified
    file_path: String,
    /// Reader of the file (Error if is an nonexistent file)
    reader: Result<BufReader<File>, String>,
    /// Whether we have read all of `self.reader`
    eof_reached: bool,
    /// Represents each line of the editor, and how many characters are in that line
    lines: Vec<usize>,
    /// Row cursor is at (1-indexed)
    row: usize,
    /// Column cursor is at (1-indexed)
    col: usize,
    /// Column cursor we want (1-indexed). When we move vertically, from a long 
    /// line to short one, we want to try to get to a specific column
    col_want: usize,
}

impl Editor {
    /// Initialize a new editor from a file path (read a single line)
    pub(crate) fn new(file_path: String) -> Editor {
        // let (reader, eof_reached) = create_reader(file_path);
        let reader;
        let eof_reached;
        if file_path == "" {
            reader = Err("No file specified".to_string());
            eof_reached = true;
        } else if Path::new(&file_path).is_file() {
            reader = Ok(BufReader::new(File::open(file_path.clone()).unwrap()));
            eof_reached = false;
        } else if Path::new(&file_path).is_dir() || file_path.ends_with("/") {
            panic!("No support (yet) for writing to directories");
        } else {
            reader = Err("File doesn't exist".to_string());
            eof_reached = true;
        }
        let mut editor = Editor {piece_table: PieceTable::new(),
            pt_index: 0,
            file_path: file_path,
            reader: reader,
            eof_reached: eof_reached,
            lines: Vec::new(),
            row: 1,
            col: 1,
            col_want: 1,
        };
        if editor.read_lines(1) == 0 {
            editor.lines.push(0);
        } else {
            editor.lines.push(editor.piece_table.text_len());
        }
        editor
    }

    /// Returns visible text
    fn text(&mut self) -> &str {
        self.piece_table.text()
    }

    /// Adds `text` at the current cursor position
    fn add_text(&mut self, text: String) {
        let mut num_lines = 0;
        let mut last_line_len = self.col_want - 1;
        for (i, line) in text.split("\n").enumerate() {
            // TODO: Insert text to visual editor
            if self.row + i - 1 >= self.lines.len() {
                self.lines.push(line.len());
            } else if i == 0 {
                *(self.lines.get_mut(self.row + i - 1).unwrap()) += line.len();
            } else {
                self.lines.insert(self.row + i - 1, line.len());
            }
            num_lines += 1;
            last_line_len = line.len();
        }
        self.piece_table.add_text(text, self.pt_index);
        if num_lines == 1 {
            self.right(last_line_len).unwrap();
        } else {
            self.down(num_lines - 1).unwrap();
            self.goto_col(last_line_len + 1).unwrap();
            println!("{:?}", self.lines);
        }
    }

    /// Read `num_lines` from `reader`, updating `self.piece_table` & `self.lines`
    /// Returns number of lines actually read
    fn read_lines(&mut self, num_lines: usize) -> usize {
        if self.eof_reached {
            return 0;
        }
        let mut lines_read = 0;
        let reader = self.reader.as_mut().unwrap();
        for _ in 0..num_lines {
            let mut temp_str = String::new();
            match reader.read_line(&mut temp_str) {
                Ok(0) => {
                    self.eof_reached = true;
                    break
                },
                Ok(len) => {
                    lines_read += 1;
                    self.lines.push(len);
                    self.piece_table.update_original_buffer(temp_str);
                },
                Err(e) => panic!("Error reading file: {:?}", e),
            }
        }
        lines_read
    }

    /// Read to EOF, updating `self.piece_table` & `self.lines`
    fn read_to_eof(&mut self) {
        // Maybe use self.read_lines(usize::MAX) instead?
        if self.eof_reached {
            return;
        }
        let reader = self.reader.as_mut().unwrap();
        loop {
            let mut temp_str = String::new();
            match reader.read_line(&mut temp_str) {
                Ok(0) => {
                    self.eof_reached = true;
                    break
                },
                Ok(len) => {
                    self.lines.push(len);
                    self.piece_table.update_original_buffer(temp_str);
                },
                Err(e) => panic!("Error reading file: {:?}", e),
            }
        }
    }

    /// Move the cursor up `num` places
    pub(crate) fn up(&mut self, num: usize) -> Result<(), String> {
        if self.row == 1 || num >= self.row {
            return Err("Can't go up".to_string());
        }
        self.pt_index -= self.col + 1;
        for i in 1..num {
            self.pt_index -= self.lines.get(self.row - i).unwrap() + 1;
        }
        self.row -= num;
        let line_cols = self.lines.get(self.row - 1).unwrap();
        self.col = min(self.col_want, line_cols + 1);
        self.pt_index -= line_cols + 1 - self.col;
        Ok(())
    }

    /// Move the cursor down `num` places
    pub(crate) fn down(&mut self, num: usize) -> Result<(), String> {
        if self.row + num > self.lines.len() {
            let from_bottom = self.row + num - self.lines.len();
            let lines_read = self.read_lines(from_bottom);
            for _ in lines_read..from_bottom {
                self.lines.push(0);
            }
        }
        self.pt_index += self.lines.get(self.row - 1).unwrap() + 1 - self.col + 1;
        for i in 1..num {
            self.pt_index += self.lines.get(self.row + i - 1).unwrap() + 1;
        }
        self.row += num;
        self.col = min(self.col_want, self.lines.get(self.row - 1).unwrap() + 1);
        self.pt_index += self.col - 1;
        Ok(())
    }

    /// Move the cursor right `num` places
    pub(crate) fn right(&mut self, num: usize) -> Result<(), String> {
        let line_cols = self.lines.get(self.row - 1).unwrap() + 1;
        if self.col + num > line_cols {
            return Err("Can't go right".to_string());
        }
        self.col += num;
        self.pt_index += num;
        self.col_want = self.col;
        Ok(())
    }

    /// Move the cursor left `num` places
    pub(crate) fn left(&mut self, num: usize) -> Result<(), String> {
        if num >= self.col {
            return Err("Can't go left".to_string());
        }
        self.col -= num;
        self.pt_index -= num;
        self.col_want = self.col;
        Ok(())
    }

    /// Move to a certain column in the current row
    pub(crate) fn goto_col(&mut self, col: usize) -> Result<(), String> {
        if col > *self.lines.get(self.row - 1).unwrap() + 1 {
            return Err("col greater than columns in row".to_string());
        }
        if self.col == col {
            Ok(())
        } else if col < self.col {
            self.left(self.col - col)
        } else {
            self.right(col - self.col)
        }
    }

    /// Move to a certain row
    pub(crate) fn goto_row(&mut self, row: usize) -> Result<(), String> {
        if self.row == row {
            Ok(())
        } else if row < self.row {
            self.up(self.row - row)
        } else {
            self.down(row - self.row)
        }
    }

    /// Move to the last column in the current row
    pub(crate) fn goto_last_col(&mut self) -> Result<(), String> {
        self.goto_col(*self.lines.get(self.row - 1).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_text() {
        let mut editor = Editor::new("".to_string());
        let mut want_str = "hello";
        editor.add_text(want_str.to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new("".to_string());
        want_str = "hello\nbye";
        editor.add_text(want_str.to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new("".to_string());
        editor.add_text("hello\n".to_string());
        editor.add_text("bye".to_string());
        want_str = "hello\nbye";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new("".to_string());
        editor.add_text("hello\n\n".to_string());
        editor.add_text("bye".to_string());
        want_str = "hello\n\nbye";
        assert_eq!(editor.text(), want_str);
    }

    fn movement() {
        let mut editor = Editor::new("".to_string());
        let mut want_str = "hello\nbye";
        editor.add_text("hello".to_string());
        editor.down(1).unwrap();
        editor.add_text("bye".to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new("".to_string());
        want_str = "hello\nbye";
        editor.add_text("h".to_string());
        editor.down(1).unwrap();
        editor.add_text("bye".to_string());
        editor.up(1).unwrap();
        editor.add_text("ello".to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new("".to_string());
        want_str = "ab\nabcd";
        editor.add_text("ab".to_string());
        editor.down(1).unwrap();
        editor.add_text("abc".to_string());
        editor.up(1).unwrap();
        editor.down(1).unwrap();
        editor.add_text("d".to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new("".to_string());
        want_str = "abcde\na\n\na";
        editor.add_text("acd".to_string());
        editor.down(1).unwrap();
        editor.add_text("a".to_string());
        editor.up(1).unwrap();
        editor.add_text("b".to_string());
        editor.goto_last_col().unwrap();
        editor.add_text("e".to_string());
        editor.down(3).unwrap();
        editor.add_text("a".to_string());
        assert_eq!(editor.text(), want_str);
    }
}

/*
/// Returns a reader for a file and if a temp file had to be created
// fn create_reader(file_path: String) -> Result<(BufReader<File>, bool), String> {
fn create_reader(file_path: String) -> (Result<BufReader<File>, String>, bool) {
    // Only will work in Linux/MacOS systems
    let tmp_path;
    if file_path.starts_with("/") {
        tmp_path = "/tmp/via/".to_string() + &file_path.replace("/", "%");
    } else {
        let curr_dir = env::current_dir().unwrap().to_str().unwrap();
        tmp_path = "/tmp/via/".to_string() + &curr_dir.replace("/", "%") + &file_path.replace("/", "%");
    }
    if Path::new(&file_path).is_file() {
        // return Ok((BufReader::new(File::open(file_path).unwrap()), false))
        return (Ok(BufReader::new(File::open(file_path).unwrap())), false);
    } else if !Path::new(&tmp_path).is_file() {
        // return Ok((BufReader::new(File::open(tmp_path).unwrap()), true))
        return (Ok(BufReader::new(File::open(tmp_path).unwrap())), true);
    }
    /*
    let error_message = "File is already being edited and/or last session of Via didn't exit cleanly\n
        To remove this message, delete ".to_string() + tmp_path.as_str();
    Err(error_message.to_string())
    */
    panic!("File is already being edited and/or last session of Via didn't exit cleanly\n
    To remove this message, delete {}", tmp_path);
}
*/