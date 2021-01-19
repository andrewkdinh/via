extern crate termion;

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead, MouseTerminal};
use termion::raw::IntoRawMode;

use std::cmp::min;
use std::io::{Write, stdout, stdin};
use std::convert::TryInto;
use std::fs::File;

use super::editor::Editor;
use super::piece_table::PieceTable;

/// Via main class, comprised of `Editor`'s
pub struct Via {
    /// List of current editors
    editors: Vec<Editor>,
    /// Editor representing the command line
    cmd_editor: Editor,
    /// Configured options
    options: ViaOptions,
    /** Mode Via is currently in
     * 0: normal
     * 1: visual (not implemented)
     * 2: select (not implemented)
     * 3: insert
     * 4: command line
     * 5: ex (not implemented)
    */
    mode: usize,
    /// Piece table of the command line
    cmd_piece_table: PieceTable,
}

impl Via {
    /// Initialize a new instance of Via from arguments
    pub(crate) fn new(file_path: String, options: ViaOptions) -> Via {
        Via {
            editors: vec![Editor::new(file_path)],
            cmd_editor: Editor::new("".to_string()),
            options: options,
            mode: 0,
            cmd_piece_table: PieceTable::new(),
        }
    }

    /// Initialize Via and start editing
    pub fn init(&mut self) {
        // let stdin = termion::async_stdin();
        let stdin = stdin();
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

        write!(stdout, "{}{}{}", termion::clear::All, termion::cursor::Goto(1, 1), termion::cursor::BlinkingBlock).unwrap();
        stdout.flush().unwrap();

        // TODO: Implement support for multiple editors concurrently
        let editor = self.editors.get_mut(0).unwrap();
        let mut visual_first_row: usize = 1;

        // TODO: Remove me
        editor.goto_last_row();
        editor.goto_last_col();

        let mut full_render = false;
        for c in stdin.events() {
            let (term_rows_u16, term_cols_u16) = termion::terminal_size().unwrap();
            let term_rows: usize = term_rows_u16.try_into().unwrap();
            let _term_cols: usize = term_cols_u16.try_into().unwrap();
            let evt = c.unwrap();
            match evt {
                Event::Key(key) => {
                    if key == Key::Esc {
                        if self.mode == 4 {
                            write!(stdout, "{}", termion::cursor::Restore).unwrap();
                        } else if self.mode == 3 {
                            editor.left(1);
                        }
                        self.mode = 0;
                    } else if self.mode == 0 {
                        // Normal mode
                        match key {
                            Key::Char('h') | Key::Left | Key::Backspace => editor.left(1),
                            Key::Char('j') | Key::Down => {
                                if visual_first_row + term_rows == editor.row() {
                                    visual_first_row += 1;
                                }
                                editor.down(1);
                                if editor.col() - 1 == editor.line_len(editor.row()) {
                                    editor.left(1);
                                }
                            },
                            Key::Char('\n') => {
                                if visual_first_row + term_rows == editor.row() {
                                    visual_first_row += 1;
                                }
                                if editor.row() != editor.num_lines() {
                                    editor.goto(editor.row() + 1, 1);
                                }
                            },
                            Key::Char('k') | Key::Up => {
                                editor.up(1);
                                if editor.col() - 1 == editor.line_len(editor.row()) {
                                    editor.left(1);
                                }
                            },
                            Key::Char('l') | Key:: Right => {
                                if editor.col() < editor.line_len(editor.row()) {
                                    editor.right(1)
                                }
                            },
                            Key::Char('i') => self.mode = 3,
                            Key::Char('a') => {
                                editor.right(1); 
                                self.mode = 3
                            },
                            Key::Char('o') => {
                                editor.goto_last_col();
                                editor.add_text("\n".to_string());
                                self.mode = 3;
                                full_render = true;
                            },
                            Key::Char('O') => {
                                editor.goto_col(0);
                                editor.add_text("\n".to_string());
                                editor.up(1);
                                self.mode = 3;
                                full_render = true;
                            },
                            Key::Char(':') => {
                                write!(stdout, "{}", termion::cursor::Save).unwrap();
                                self.cmd_editor.delete_all();
                                self.cmd_editor.add_text(":".to_string());
                                self.mode = 4
                            },
                            Key::Char('$') => {
                                editor.goto_last_col();
                                editor.left(1);
                            },
                            Key::Char('0') => {
                                editor.goto_col(0);
                            },
                            Key::Char('A') => {
                                editor.goto_last_col();
                                self.mode = 3;
                            }
                            Key::Delete => {
                                let at_line_end = editor.col() == editor.num_cols(editor.row());
                                if at_line_end && editor.row() == editor.num_lines() {
                                    break
                                } else if at_line_end {
                                    editor.delete_text(editor.row() + 1, 1).unwrap();
                                    full_render = true;
                                } else {
                                    editor.delete_text(editor.row(), editor.col() + 1).unwrap();
                                    write!(stdout, "{}{}", termion::clear::CurrentLine, editor.text_line(editor.row())).unwrap();
                                }
                            },
                            Key::Home => editor.goto_col(0),
                            Key::End => editor.goto_last_col(),
                            _ => {},
                        }
                    } else if self.mode == 3 {
                        // Insert mode
                        match key {
                            Key::Char(c) => {
                                if c == '\n' {
                                    editor.add_text(c.to_string());
                                    full_render = true;
                                } else {
                                    editor.add_text(c.to_string());
                                    write!(stdout, "{}{}", "\r", editor.text_line(editor.row())).unwrap();
                                }
                            },
                            Key::Left => editor.left(1),
                            Key::Down => editor.down(1),
                            Key::Up => editor.up(1),
                            Key::Right => editor.right(1),
                            Key::Backspace => {
                                if editor.row() == 1 && editor.col() == 1 {
                                    
                                } else if editor.col() == 1 {
                                    editor.up(1);
                                    editor.goto_last_col();
                                    editor.delete_text(editor.row() + 1, 1).unwrap();
                                    full_render = true;
                                } else {
                                    editor.left(1);
                                    editor.delete_text(editor.row(), editor.col() + 1).unwrap();
                                    write!(stdout, "{}{}{}", termion::clear::CurrentLine, termion::cursor::Goto(1, (editor.row() - visual_first_row + 1).try_into().unwrap()), editor.text_line(editor.row())).unwrap();
                                }
                            },
                            Key::Delete => {
                                let at_line_end = editor.col() == editor.num_cols(editor.row());
                                if at_line_end && editor.row() == editor.num_lines() {
                                    break
                                } else if at_line_end {
                                    editor.delete_text(editor.row() + 1, 1).unwrap();
                                    full_render = true;
                                } else {
                                    editor.delete_text(editor.row(), editor.col() + 1).unwrap();
                                    write!(stdout, "{}{}", termion::clear::CurrentLine, editor.text_line(editor.row())).unwrap();
                                }
                            },
                            Key::Home => editor.goto_col(0),
                            Key::End => editor.goto_last_col(),
                            _ => {}
                        }
                    } else if self.mode == 4 {
                        // Command line mode
                        match key {
                            Key::Char('\n') => {
                                let mut write = false;
                                let mut skip_write = false;
                                let mut quit = false;
                                match self.cmd_editor.text() {
                                    ":wq" | "x" => {write = true; quit = true;},
                                    ":q" => quit = true,
                                    ":q!" => {skip_write = true; quit = true}
                                    text => {
                                        // Don't panic, but output an error message
                                        panic!(text.to_string())
                                    },
                                }
                                if write {
                                    if editor.file_path() == "" {
                                        // Don't panic, but output an error message
                                        panic!("FIXME")
                                    } else {
                                        File::create(editor.file_path()).unwrap().write_all(editor.text().as_bytes()).unwrap();
                                    }
                                }
                                if quit {
                                    if editor.text_matches() {
                                        write!(stdout, "{}", termion::cursor::BlinkingBlock).unwrap();
                                        break
                                    } else if skip_write {
                                        break
                                    }
                                    // Don't panic, but output an error message
                                    panic!("Quit without saving")
                                }
                                panic!("Invalid command")
                            }
                            Key::Char(c) => {
                                self.cmd_editor.add_text(c.to_string());
                            },
                            Key::Left => self.cmd_editor.left(1),
                            Key::Right => self.cmd_editor.right(1),
                            Key::Backspace => {
                                if self.cmd_editor.col() != 1 {
                                    self.cmd_editor.left(1);
                                    self.cmd_editor.delete_text(self.cmd_editor.row(), self.cmd_editor.col() + 1).unwrap();
                                }
                            },
                            Key::Delete => {
                                let at_line_end = self.cmd_editor.col() == self.cmd_editor.num_cols(self.cmd_editor.row());
                                if !at_line_end {
                                    self.cmd_editor.delete_text(self.cmd_editor.row(), self.cmd_editor.col() + 1).unwrap();
                                    write!(stdout, "{}{}", termion::clear::CurrentLine, self.cmd_editor.text_line(self.cmd_editor.row())).unwrap();
                                }
                            },
                            Key::Home => self.cmd_editor.goto_col(0),
                            Key::End => self.cmd_editor.goto_last_col(),
                            _ => {}
                        }
                    } else {
                        panic!("Mode {} not implemented yet", self.mode);
                    }
                },
                Event::Mouse(me) => {
                    match me {
                        MouseEvent::Press(_, x, y) => {
                            editor.goto(y.try_into().unwrap(), min(x.try_into().unwrap(), editor.line_len(editor.row())));
                            self.mode = 0;
                        },
                        _ => (),
                    }
                }
                _ => {}
            }
            if full_render {
                // write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
                for (i, line) in editor.text_lines(visual_first_row, visual_first_row + min(editor.num_lines(), term_rows)).lines().enumerate() {
                    write!(stdout, "{}{}", termion::cursor::Goto(1, (i + 1).try_into().unwrap()), line).unwrap();
                }
                full_render = false;
            }
            write!(stdout, "{}", termion::cursor::Goto(editor.col().try_into().unwrap(), (editor.row() - visual_first_row + 1).try_into().unwrap())).unwrap();
            if self.mode == 0 {
                write!(stdout, "{}", termion::cursor::BlinkingBlock).unwrap();
            } else {
                write!(stdout, "{}", termion::cursor::BlinkingBar).unwrap();
            }
            stdout.flush().unwrap();
        }
        write!(stdout, "{}{}{}{}", termion::clear::All, termion::cursor::Goto(1, 1), termion::cursor::Show, termion::cursor::BlinkingBlock).unwrap();
    }
    
    /// Process command line options and returns the files to edit and ViaOptions
    // fn process_args(&mut self, args: &Vec<String>) -> Result<(Vec<String>, ViaOptions), &str> {
    pub(crate) fn process_args(args: Vec<String>) -> (Vec<String>, ViaOptions) {
        let mut flags = Vec::new();
        let mut file_paths = Vec::new();
        // let default_files = vec!["target/debug/via", "via"];
        for arg in &args[1..] {
            if arg == "--" {
                break
            } else if arg.starts_with("-") {
                flags.push(arg);
            } else {
                file_paths.push((*arg).as_str().to_string());
            }
        }
        let mut via_options = ViaOptions::new();
        for option in flags {
            if option == "-v" {
                via_options.verboseness += 1;
            } else {
                panic!("Unknown flag {}", option);
            }
        }
        (file_paths, via_options)
    }
}

#[derive(Clone)]
pub(crate) struct ViaOptions {
    /// Level of verboseness
    verboseness: usize,
}

impl ViaOptions {
    /// Return default options
    pub(crate) fn new() -> ViaOptions {
        ViaOptions {verboseness: 1}
    }
}