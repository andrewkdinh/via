// use unicode_segmentation::UnicodeSegmentation;

/// The main structure for storing text
pub(crate) struct PieceTable {
    /// The main table, contains `TableEntry`'s
    table: Vec<TableEntry>,
    /// Original buffer
    original_buffer: String,
    /// Add buffer
    add_buffer: String,
    /// All active text. Only to be used when `text_up_to_date == true`
    text: String,
    /// Length of text if it was up to date (`text` may not not be up to date)
    text_len: usize,
    /// Whether `text` is up to date
    text_up_to_date: bool,
    /// List of actions, which are lists of table entries' indices
    actions: Vec<Vec<usize>>,
    /// Where in `self.actions` we are currently at
    /// 
    /// **NOTE**: A value of 0 means no actions have been taken
    actions_index: usize,
}

impl PieceTable {
    /// Initializes a piece table
    pub(crate) fn new() -> PieceTable {
        PieceTable {
            table: Vec::new(),
            original_buffer: String::new(),
            add_buffer: String::new(),
            text: String::new(),
            text_len: 0,
            text_up_to_date: true,
            actions: Vec::new(),
            actions_index: 0,
        }
    }

    /// Returns if any actions have been taken
    pub(crate) fn actions_taken(&self) -> bool {
        !self.actions.is_empty()
    }

    /// Append text to the original buffer and add a table entry
    pub(crate) fn update_original_buffer(&mut self, text: String) {
        let org_buffer_len = self.original_buffer.len();
        self.original_buffer.push_str(&text);
        self.table.push(TableEntry::new(false, org_buffer_len, org_buffer_len + text.len()));
        self.text_len += text.len();
        self.text_up_to_date = false;
    }

    /// Add text at a certain index
    pub(crate) fn add_text(&mut self, text: String, index: usize) {
        let text_len = text.len();
        let add_buffer_len = self.add_buffer.len();
        if index > self.text_len {
            panic!("index ({}) is a greater value than text len ({})", index, self.text_len);
        }
        let mut curr_pos = 0;
        let mut action: Vec<usize> = Vec::new();
        let mut add_table: Vec<TableEntry> = Vec::with_capacity(3);
        let mut add_table_indices: Vec<usize> = Vec::with_capacity(3);
        if index == 0 {
            self.table.insert(0, TableEntry::new(true, add_buffer_len, add_buffer_len + text.len()));
            action.push(0);
        } else if index == self.text_len {
            self.table.push(TableEntry::new(true, add_buffer_len, add_buffer_len + text.len()));
            action.push(self.table.len());
        } else {
            for (i, table_entry) in self.table.iter_mut().enumerate() {
                if table_entry.active {
                    let len = table_entry.end_index - table_entry.start_index;
                    if curr_pos == index {
                        add_table.push(TableEntry::new(true, add_buffer_len, add_buffer_len + text.len()));
                        add_table_indices.push(i);
                        break
                    } else if curr_pos + len > index {
                        // Split into 2 parts and disable original [ab] + [c] -> [a][c][b]
                        let split_point = index - curr_pos;

                        table_entry.active = false;
                        action.push(i);

                        add_table.push(TableEntry::new(table_entry.is_add_buffer, table_entry.start_index, table_entry.start_index + split_point));
                        action.push(i + 1);
                        add_table.push(TableEntry::new(true, add_buffer_len, add_buffer_len + text.len()));
                        action.push(i + 2);
                        add_table.push(TableEntry::new(table_entry.is_add_buffer, table_entry.start_index + split_point, table_entry.end_index));
                        action.push(i + 3);
    
                        add_table_indices.push(i);
                        add_table_indices.push(i + 1);
                        add_table_indices.push(i + 2);
                        break
                    }
                    curr_pos += len;
                }
            }
    
            for (i, table_entry) in add_table_indices.iter().zip(add_table) {
                self.table.insert(*i, table_entry);
            }
        }

        self.add_buffer.push_str(&text);
        self.add_action(action);
        self.text_len += text_len;
        self.text_up_to_date = false;
    }

    /// Delete text from `start` to `end`
    pub(crate) fn delete_text(&mut self, start: usize, end: usize) {
        if start >= end || end == 0 || end > self.text_len  {
            panic!("Can't delete from start ({}) to end ({}) of text size {}", start, end, self.text_len);
        }
        let mut curr_pos = 0;
        let mut action: Vec<usize> = Vec::new();
        let mut add_table: Vec<TableEntry> = Vec::with_capacity(4);
        let mut add_table_indices: Vec<usize> = Vec::with_capacity(4);

        for (i, table_entry) in self.table.iter_mut().enumerate() {
            if curr_pos == end {
                break
            }
            if table_entry.active {
                let len = table_entry.end_index - table_entry.start_index;
                if start <= curr_pos && end >= curr_pos + len {
                    // At table entry to continue/end at
                    // OR start & end is this exact table entry
                    table_entry.active = false;
                    action.push(i + add_table.len());
                } else if start > curr_pos && start < curr_pos + len && end >= curr_pos + len {
                    // At table entry to start at (split table entry in two)
                    // Only occurs once, and will be the first
                    let split_point = start - curr_pos;
                    
                    table_entry.active = false;
                    action.push(i);

                    add_table.push(TableEntry::new(table_entry.is_add_buffer, table_entry.start_index, table_entry.start_index + split_point));
                    add_table_indices.push(i + 1);
                    action.push(i + 1);
                } else if start <= curr_pos && end > curr_pos && end < curr_pos + len {
                    // At table entry to end at (split table entry in two)
                    let split_point = end - curr_pos;

                    table_entry.active = false;
                    action.push(i + add_table.len());

                    add_table.push(TableEntry::new(table_entry.is_add_buffer, table_entry.start_index + split_point, table_entry.end_index));
                    add_table_indices.push(i + add_table.len());
                    action.push(i + add_table.len());
                    break
                } else if start > curr_pos && end < curr_pos + len {
                    // At table entry to split into 3 [abc] -> [a](b)[c]
                    // Only occurs once, and will be the first/last
                    let first_split = start - curr_pos;
                    let second_split = end - curr_pos;

                    table_entry.active = false;
                    action.push(i);

                    add_table.push(TableEntry::new(table_entry.is_add_buffer, table_entry.start_index, table_entry.start_index + first_split));
                    add_table_indices.push(i + 1);
                    action.push(i + 1);

                    add_table.push(TableEntry::new(table_entry.is_add_buffer, table_entry.start_index + second_split, table_entry.end_index));
                    add_table_indices.push(i + 2);
                    action.push(i + 2);
                    break
                }
                curr_pos += len;
            }
        }

        for (i, table_entry) in add_table_indices.iter().zip(add_table) {
            self.table.insert(*i, table_entry);
        }

        self.add_action(action);
        self.text_len -= end - start;
        self.text_up_to_date = false;
    }

    /// Add a table entry to actions
    fn add_action(&mut self, action: Vec<usize>) {
        // Remove actions after current index
        // TODO: Remove all unecessary TableEntry's
        self.actions = self.actions[..self.actions_index].to_vec();
        self.actions.push(action);
        self.actions_index = self.actions.len();
    }

    /// Undo an action. Errors if no actions to undo
    fn undo(&mut self) {
        if self.actions.is_empty() {
            panic!("Unable to undo");
        }

        for index in self.actions.get(self.actions_index - 1).unwrap() {
            let table_entry = self.table.get_mut(*index).unwrap();
            table_entry.switch();
            if table_entry.active {
                self.text_len += table_entry.end_index - table_entry.start_index;
            } else {
                self.text_len -= table_entry.end_index - table_entry.start_index;
            }
        }

        self.text_up_to_date = false;
        self.actions_index -= 1;
    }

    /// Redo an action. Errors if no actions to redo
    fn redo(&mut self) {
        if self.actions.is_empty() || self.actions.len() <= self.actions_index {
            panic!("Unable to redo");
        }
        for index in self.actions.get(self.actions_index).unwrap() {
            let table_entry = self.table.get_mut(*index).unwrap();
            table_entry.switch();
            if table_entry.active {
                self.text_len += table_entry.end_index - table_entry.start_index;
            } else {
                self.text_len -= table_entry.end_index - table_entry.start_index;
            }
        }
        self.text_up_to_date = false;
        self.actions_index += 1;
    }

    /// Returns the text represented by a table entry
    fn table_entry_text(&self, table_entry: &TableEntry) -> &str {
        let buffer = if table_entry.is_add_buffer {&self.add_buffer} else {&self.original_buffer};
        buffer.get(table_entry.start_index..table_entry.end_index).unwrap()
    }

    /// Returns length of text
    pub(crate) fn text_len(&self) -> usize {
        self.text_len
    }

    /// Returns all visible text
    /// 
    /// If you want to get the length of the text, use `text_len(&self)` instead
    pub(crate) fn text(&mut self) -> &str {
        self.update_text();
        &self.text
    }

    /// Updates all text for the piece table
    fn update_text(&mut self) {
        if self.text_up_to_date {
            return;
        }

        let mut text = String::new();
        for table_entry in &self.table {
            if table_entry.active {
                text.push_str(self.table_entry_text(table_entry));
            }
        }
        
        self.text = text;
        self.text_up_to_date = true;
    }

    /// Insert a table entry to a specific index
    fn insert(&mut self, index: usize, table_entry: TableEntry) {
        self.table.insert(index, table_entry);
        self.text_up_to_date = false;
    }

    /// Add a table entry to the end of the table
    fn push(&mut self, table_entry: TableEntry) {
        self.table.push(table_entry);
        self.text_up_to_date = false;
    }

    /// Return the index-th line
    /// TODO: Remove this and do something better regarding lines
    pub(crate) fn line(&mut self, index: usize) -> Result<&str, &str> {
        for (i, line) in self.text().lines().enumerate() {
            if i == index {
                return Ok(line)
            }
        }
        Err("Invalid line index")
    }
}

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
    pub(crate) fn new(is_add_buffer: bool, start_index: usize, end_index: usize) -> TableEntry {
        TableEntry {
            is_add_buffer: is_add_buffer, 
            start_index: start_index, 
            end_index: end_index, 
            active: true,
        }
    }

    /// Change from active to deactivated and visa versa
    fn switch(&mut self) {
        self.active = !self.active;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_text() {
        let mut piece_table = PieceTable::new();
        let mut want_str = "a";
        piece_table.add_text("a".to_string(), 0);
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table.add_text(" b".to_string(), 1);
        want_str = "a b";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table.add_text("c ".to_string(), 2);
        want_str = "a c b";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table.add_text("d ".to_string(), 0);
        want_str = "d a c b";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);
    }

    #[test]
    fn delete_text() {
        let mut piece_table = PieceTable::new();
        piece_table.add_text("abc".to_string(), 0);
        piece_table.delete_text(0, 1);
        let mut want_str = "bc";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("abc".to_string(), 0);
        piece_table.delete_text(1, 2);
        want_str = "ac";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("abc".to_string(), 0);
        piece_table.delete_text(2, 3);
        want_str = "ab";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("abc".to_string(), 0);
        piece_table.delete_text(2, 3);
        want_str = "ab";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("ab".to_string(), 0);
        piece_table.add_text("cd".to_string(), 2);
        piece_table.delete_text(2, 3);
        want_str = "abd";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("ab".to_string(), 0);
        piece_table.add_text("cd".to_string(), 2);
        piece_table.add_text("ef".to_string(), 4);
        piece_table.delete_text(1, 5);
        want_str = "af";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("ab".to_string(), 0);
        piece_table.add_text("cd".to_string(), 2);
        piece_table.add_text("ef".to_string(), 4);
        piece_table.delete_text(1, 6);
        want_str = "a";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("ab".to_string(), 0);
        piece_table.add_text("cd".to_string(), 2);
        piece_table.add_text("ef".to_string(), 4);
        piece_table.delete_text(0, 6);
        want_str = "";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);
    }

    #[test]
    fn undo_redo() {
        let mut piece_table = PieceTable::new();
        piece_table.add_text("abc".to_string(), 0);
        piece_table.delete_text(0, 1);
        let mut want_str = "abc";
        piece_table.undo();
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table.redo();
        want_str = "bc";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("abc".to_string(), 0);
        piece_table.add_text("d".to_string(), 3); // "abcd"
        piece_table.delete_text(0, 2); // "cd"
        piece_table.undo();
        want_str = "abcd";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);
        piece_table.undo();
        want_str = "abc";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);
        piece_table.redo();
        want_str = "abcd";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);
        piece_table.redo();
        want_str = "cd";
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);
    }

    #[test]
    fn edge_cases() {
        let mut piece_table = PieceTable::new();
        piece_table.add_text("\n".to_string(), 0);
        let mut want_str = "\n";
        assert_eq!(want_str.len(), 1);
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("\n\n\n\n".to_string(), 0);
        want_str = "\n\n\n\n";
        assert_eq!(want_str.len(), 4);
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        // TODO: Add support for graphemes
        // https://stackoverflow.com/a/46290728
        piece_table = PieceTable::new();
        piece_table.add_text("😀".to_string(), 0);
        want_str = "😀";
        assert_eq!(want_str.len(), 4);
        // assert_eq!(want_str.graphemes(true).count(), 1);
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);

        piece_table = PieceTable::new();
        piece_table.add_text("é".to_string(), 0);
        want_str = "é";
        assert_eq!(want_str.len(), 3);
        assert_eq!(want_str.chars().count(), 2);
        // assert_eq!(want_str.graphemes(true).count(), 1);
        assert_eq!(piece_table.text_len, want_str.len());
        assert_eq!(piece_table.text(), want_str);
    }
}