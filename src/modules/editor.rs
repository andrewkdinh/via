use std::fs::File;
use std::io::{BufRead, BufReader};
use std::cmp::{min, max};
use std::path::Path;

use super::piece_table::PieceTable;

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
    pub(crate) fn text(&mut self) -> &str {
        self.piece_table.text()
    }

    /// Returns file path
    pub(crate) fn file_path(&self) -> &str {
        self.file_path.as_str()
    }

    /// Update the file path
    pub(crate) fn update_file_path(&mut self, file_path: String) {
        self.file_path = file_path;
    }

    /// Returns the current row
    pub(crate) fn row(&self) -> usize {
        self.row
    }

    /// Returns the current column
    pub(crate) fn col(&self) -> usize {
        self.col
    }

    /// Returns the number of columns in the specified `row` (1-indexed)
    pub(crate) fn num_cols(&self, row: usize) -> usize {
        *self.lines.get(row - 1).unwrap()
    }

    /// Returns the number of lines
    pub(crate) fn num_lines(&self) -> usize {
        self.lines.len()
    }

    /// Returns the length of the line (1-indexed)
    pub(crate) fn line_len(&self, line: usize) -> usize {
        *self.lines.get(line - 1).unwrap()
    }

    /// Returns whether the text of the file matches the text of `self.piece_table`
    pub(crate) fn text_matches(&self) -> bool {
        !self.piece_table.actions_taken()
    }

    /// Returns visible text from line `first` (inclusive) to `last` (exclusive)
    pub(crate) fn text_lines(&mut self, first: usize, last: usize) -> &str {
        if first >= last {
            panic!("First row ({}) must be less last ({})", first, last);
        }
        let mut start_index = 0;
        let mut end_index = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if i < first - 1 {
                start_index += line + 1;
                end_index = start_index;
            } else if i >= last - 1 {
                break
            } else {
                end_index += line + 1;
            }
        }
        self.piece_table.text().get(start_index..end_index - 1).unwrap()
    }

    /// Returns the visible text for a single row
    pub(crate) fn text_line(&mut self, line: usize) -> &str {
        self.text_lines(line, line + 1)
    }

    /// Adds `text` at the current cursor position
    pub(crate) fn add_text(&mut self, text: String) {
        let mut from_end = 0;
        let mut num_lines = 0;
        let text_len = text.len();
        let mut last_line_len = 0;
        for (i, line) in text.split('\n').enumerate() {
            if i == 0 {
                let curr_line_len = self.lines.get_mut(self.row - 1).unwrap();
                from_end = *curr_line_len + 1 - self.col;
                if text.contains('\n') {
                    *curr_line_len -= from_end;
                }
                *curr_line_len += line.len();
            } else if self.row + i >= self.lines.len() {
                self.lines.push(line.len() + from_end);
                from_end = 0;
            } else {
                self.lines.insert(self.row + i, line.len() + from_end);
                from_end = 0;
            }
            num_lines += 1;
            last_line_len = line.len();
        }
        self.piece_table.add_text(text, self.pt_index);
        if num_lines == 1 {
            self.right(text_len);
        } else {
            self.down(num_lines - 1);
            self.goto_col(last_line_len + 1);
        }
    }

    /// Deletes from current cursor position to (row, col) which are 1-indexed
    pub(crate) fn delete_text(&mut self, row: usize, col: usize) -> Result<(), String> {
        if row == self.row {
            if row == self.row && col == self.col {
                return Ok(())
                // return Err("No text to delete".to_string());
            }
            let line_len = self.lines.get_mut(row - 1).unwrap();
            let first_col = min(self.col, col);
            let last_col = if first_col == self.col {col} else {self.col};
            if last_col > *line_len + 1 {
                // panic!("Can't delete from {} to {} of line length {}", first_col, last_col, *line_len);
                return Err(format!("Can't delete from {} to {} of line length {}", first_col, last_col, *line_len))
            }
            let len = last_col - first_col;
            *(line_len) -= len;
            if first_col == self.col {
                self.piece_table.delete_text(self.pt_index, self.pt_index + len);
            } else {
                self.piece_table.delete_text(self.pt_index - len, self.pt_index);
                self.col -= len;
                self.col_want = self.col;
            }
            return Ok(())
        }

        let mut size = 0;
        let first_row = min(self.row, row);
        let first_col = if first_row == self.row {self.col} else {col};
        let last_row = if first_row == self.row {row} else {self.row};
        let last_col = if first_row == self.row {col} else {self.col};

        if last_row == usize::MAX {
            // TODO: Don't actually read to end of file. Just pretend you did
            // If you do this, you have to update undo and redo to update self.eof_reached
            self.read_to_eof()
        } else {
            self.read_lines(max(self.lines.len(), row) - min(self.lines.len(), row));
        }
        
        let first_line_len = self.lines.get_mut(first_row - 1).unwrap();
        if first_col > *first_line_len + 1 {
            panic!("Invalid beginning column {} for row {}", first_col, first_row);
        }
        size += *first_line_len + 1 - (first_col - 1);
        *first_line_len -= *first_line_len - (first_col - 1);
        let first_line_len_copy = *first_line_len;
        let to_last_row = last_row == self.lines.len();
        for _ in first_row + 1..last_row {
            let line_len = self.lines.remove(first_row);
            size += line_len + 1;
        }
        let last_line_len = self.lines.get_mut(last_row - (last_row - first_row) - 1).unwrap();
        if last_col - 1 > *last_line_len {
            panic!("Invalid ending column {} for row {}", last_col, last_row);
        }
        *(last_line_len) -= last_col - 1;
        size += last_col - 1;
        if to_last_row {
            self.lines.pop();
        }
        if first_row == self.row {
            self.piece_table.delete_text(self.pt_index, self.pt_index + size);
        } else {
            self.piece_table.delete_text(self.pt_index - size, self.pt_index);
            self.row = first_row;
            self.col = first_line_len_copy;
            self.col_want = self.col;
        }
        Ok(())
    }

    /// Delete all text
    pub(crate) fn delete_all(&mut self) {
        if self.piece_table.text_len() == 0 {
            return
        }
        self.piece_table.delete_text(0, self.piece_table.text_len());
        self.row = 1;
        self.col = 1;
        self.col_want = 1;
        self.pt_index = 0;
    }

    pub(crate) fn delete_to_end(&mut self) {
        self.delete_text(usize::MAX, usize::MAX).unwrap();
    }

    /// Read `num_lines` from `reader`, updating `self.piece_table` & `self.lines`
    /// Returns number of lines actually read
    fn read_lines(&mut self, num_lines: usize) -> usize {
        if num_lines == 0 || self.eof_reached {
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
    /// If unable to go up all the way, go to first row
    pub(crate) fn up(&mut self, num: usize) {
        if num == 0 || self.row == 1 {
            return
        } else if num >= self.row {
            self.up(self.row - 1);
            return
        }
        self.pt_index -= self.col;
        for i in 1..num {
            self.pt_index -= self.lines.get(self.row - i - 1).unwrap() + 1;
        }
        self.row -= num;
        let line_cols = self.lines.get(self.row - 1).unwrap();
        self.col = min(self.col_want, line_cols + 1);
        self.pt_index -= line_cols + 1 - self.col;
    }

    /// Move the cursor down `num` places.
    /// If unable to go all the way down, go to last row
    pub(crate) fn down(&mut self, num: usize) {
        if num == 0 {
            return
        } else if self.row + num > self.lines.len() {
            let lines_read = self.read_lines(self.row + num - self.lines.len());
            self.down(lines_read);
            return
        }
        self.pt_index += self.lines.get(self.row - 1).unwrap() + 1 - self.col + 1;
        for i in 1..num {
            self.pt_index += self.lines.get(self.row + i - 1).unwrap() + 1;
        }
        self.row += num;
        self.col = min(self.col_want, self.lines.get(self.row - 1).unwrap() + 1);
        self.pt_index += self.col - 1;
    }

    /// Move the cursor right `num` places.
    /// If unable to go all the way right, go to last column
    pub(crate) fn right(&mut self, num: usize) {
        let line_len = self.lines.get(self.row - 1).unwrap();
        if num == 0 || self.col == line_len + 1 {
            return
        } else if self.col + num > line_len + 1 {
            self.goto_last_col();
            return
        }
        self.col += num;
        self.pt_index += num;
        self.col_want = self.col;
    }

    /// Move the cursor left `num` places.
    /// If unable to go all the way left, go to first column
    pub(crate) fn left(&mut self, num: usize) {
        if num == 0 {
            return
        } else if num >= self.col {
            self.left(self.col - 1);
            return
        }
        self.col -= num;
        self.pt_index -= num;
        self.col_want = self.col;
    }

    /// Move to a certain column in the current row
    pub(crate) fn goto_col(&mut self, col: usize) {
        if col > *self.lines.get(self.row - 1).unwrap() + 1 {
            self.goto_last_col();
        } else if self.col == col {
            self.col_want = col;
        } else if col < self.col {
            self.left(self.col - col)
        } else {
            self.right(col - self.col)
        }
    }

    /// Move to a certain row
    pub(crate) fn goto_row(&mut self, row: usize) {
        if self.row == row {
            return
        } else if row < self.row {
            self.up(self.row - row)
        } else {
            self.down(row - self.row)
        }
    }

    /// Move to (closest) `row` and `col`
    pub(crate) fn goto(&mut self, row: usize, col: usize) {
        self.goto_row(row);
        self.goto_col(col);
    }

    /// Move to the last column in the current row
    pub(crate) fn goto_last_col(&mut self) {
        self.goto_col(*self.lines.get(self.row - 1).unwrap() + 1)
    }

    /// Move to the last row
    pub(crate) fn goto_last_row(&mut self) {
        self.read_to_eof();
        self.goto(self.lines.len(), 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_text() {
        let mut editor = Editor::new(String::new());
        let mut want_str = "hello";
        editor.add_text(want_str.to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        want_str = "hello\nbye";
        editor.add_text(want_str.to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("hello\n".to_string());
        editor.add_text("bye".to_string());
        want_str = "hello\nbye";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("hello\n\n".to_string());
        editor.add_text("bye".to_string());
        want_str = "hello\n\nbye";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("hello".to_string());
        editor.add_text("\nbye".to_string());
        want_str = "hello\nbye";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("\nhello".to_string());
        editor.add_text("\nbye".to_string());
        want_str = "\nhello\nbye";
        assert_eq!(editor.text(), want_str);
    }

    #[test]
    fn delete_text() {
        let mut editor = Editor::new(String::new());
        let mut want_str = "";
        editor.add_text("abcd".to_string());
        editor.delete_text(1, 1).unwrap();
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        want_str = "ab\nef";
        editor.add_text("ab\ncd\nef".to_string());
        editor.goto(2, 1);
        editor.delete_text(3, 1).unwrap();
        assert_eq!(editor.text(), want_str);
        assert_eq!(editor.num_lines(), want_str.lines().count());

        editor = Editor::new(String::new());
        want_str = "ab\n\ncd";
        editor.add_text("ab\n\n\ncd".to_string());
        editor.goto(3, 1);
        editor.delete_text(4, 1).unwrap();
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        want_str = "ab\n\ncd";
        editor.add_text("ab\n\n\ncd".to_string());
        editor.goto(4, 1);
        editor.delete_text(3, 1).unwrap();
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("h".to_string());
        editor.add_text("\n".to_string());
        editor.add_text("b".to_string());
        want_str = "h\nb";
        assert_eq!(editor.text(), want_str);
    }

    #[test]
    fn movement() {
        let mut editor = Editor::new(String::new());
        let mut want_str = "hello\nbye";
        editor.add_text("hello\n".to_string());
        editor.down(1);
        editor.add_text("bye".to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        want_str = "hello\nbye";
        editor.add_text("h\n".to_string());
        editor.down(1);
        editor.add_text("bye".to_string());
        editor.up(1);
        editor.add_text("ello".to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        want_str = "ab\nabcd";
        editor.add_text("ab\n".to_string());
        editor.down(1);
        editor.add_text("abc".to_string());
        editor.up(1);
        editor.down(1);
        editor.add_text("d".to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        want_str = "abcde\na\n\na";
        editor.add_text("acd\n".to_string());
        editor.add_text("a\n\n".to_string());
        assert_eq!(editor.text(), "acd\na\n\n");
        editor.up(3);
        editor.right(1);
        editor.add_text("b".to_string());
        assert_eq!(editor.text(), "abcd\na\n\n");
        editor.goto_last_col();
        editor.add_text("e".to_string());
        assert_eq!(editor.text(), "abcde\na\n\n");
        editor.down(3);
        editor.add_text("a".to_string());
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("he".to_string());
        editor.left(1);
        editor.add_text("\n".to_string());
        editor.add_text("b".to_string());
        want_str = "h\nbe";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("hellobye".to_string());
        editor.left(3);
        editor.add_text("\n".to_string());
        want_str = "hello\nbye";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("helloye".to_string());
        editor.left(2);
        editor.add_text("\nb".to_string());
        want_str = "hello\nbye";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("abc".to_string());
        editor.left(2);
        editor.add_text("d\n".to_string());
        editor.add_text("e".to_string());
        want_str = "ad\nebc";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("helloe".to_string());
        editor.left(1);
        editor.add_text("\nb".to_string());
        editor.add_text("y".to_string());
        want_str = "hello\nbye";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("hellye".to_string());
        editor.left(2);
        editor.add_text("\nb".to_string());
        editor.up(1);
        editor.goto_last_col();
        editor.add_text("o".to_string());
        want_str = "hello\nbye";
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        editor.add_text("hello".to_string());
        editor.add_text("\n".to_string());
        editor.add_text("by".to_string());
        editor.up(1);
        editor.goto_last_col();
        editor.add_text("\n".to_string());
        editor.add_text("and".to_string());
        editor.down(1);
        editor.goto_last_col();
        editor.add_text("e".to_string());
        want_str = "hello\nand\nbye";
        assert_eq!(editor.text(), want_str);
    }

    #[test]
    fn edge_cases() {
        let mut editor = Editor::new(String::new());
        let mut want_str = "hell";
        editor.add_text("hello\n\n".to_string());
        editor.delete_text(1, 5).unwrap();
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        want_str = "hello";
        editor.add_text("hello\n\n".to_string());
        editor.delete_text(1, 6).unwrap();
        assert_eq!(editor.text(), want_str);

        editor = Editor::new(String::new());
        want_str = "";
        editor.add_text("\n\n".to_string());
        editor.delete_text(1, 1).unwrap();
        assert_eq!(editor.text(), want_str);
    }

    #[test]
    fn more_cases() {
        let mut editor = Editor::new(String::new());
        let mut want_str = "hello\nbye";
        editor.add_text("hello\n\n".to_string());
        editor.add_text("bye".to_string());
        editor.up(2);
        editor.goto_last_col();
        editor.delete_text(editor.row() + 1, 1).unwrap();
        assert_eq!(editor.text(), want_str);
        println!("{:?}", editor.lines);
        assert_eq!(editor.num_lines(), want_str.lines().count());

        editor = Editor::new(String::new());
        want_str = "hellobye";
        editor.add_text("hello\n\n".to_string());
        editor.add_text("bye".to_string());
        editor.up(2);
        editor.goto_last_col();
        editor.delete_text(editor.row() + 2, 1).unwrap();
        assert_eq!(editor.text(), want_str);
        assert_eq!(editor.num_lines(), want_str.lines().count());
    }

    #[test]
    fn text_lines() {
        let mut editor = Editor::new(String::new());
        let mut want_str = "abc";
        editor.add_text("abc".to_string());
        assert_eq!(editor.text_lines(1, 2), want_str);

        editor = Editor::new(String::new());
        want_str = "abc";
        editor.add_text("abc\n\ncd".to_string());
        assert_eq!(editor.text_line(1), want_str);

        editor = Editor::new(String::new());
        want_str = "abc";
        editor.add_text("abc\n\ncd".to_string());
        assert_eq!(editor.text_lines(1, 2), want_str);

        editor = Editor::new(String::new());
        want_str = "";
        editor.add_text("abc\n\ncd".to_string());
        assert_eq!(editor.text_line(2), want_str);

        editor = Editor::new(String::new());
        want_str = "\ncd\ne";
        editor.add_text("abc\n\ncd\ne".to_string());
        assert_eq!(editor.text_lines(2, 5), want_str);
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