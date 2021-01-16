#![allow(dead_code)]
// #![warn(unused_variables)]
// #![warn(unused_mut)]

use std::env;

mod modules;

use modules::via::Via;

fn main() {
    // let mut via = Via::new(env::args().collect());
    /*
    let mut editor = initialize(process_args(&args));
    // Print the current file text onto screen
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut editor_x: usize = 1;
    let mut editor_y: usize = 0;
    write!(stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)).unwrap();

    for line in editor.piece_table.text().lines() {
        editor_y += 1;
        write!(stdout,
            "{}{}",
            termion::cursor::Goto(editor_x.try_into().unwrap(), editor_y.try_into().unwrap()),
            line).unwrap();
    }
    stdout.flush().unwrap();

    let mut buffer_index = editor.piece_table.text_len();
    let mut cmd_index = 0;
    let mut cmd_piece_table = piece_table::PieceTable::new();
    let mut mode = 1; // [0: insert, 1: visual, 2: command]
    for c in stdin.keys() {
        write!(stdout,
               "{}{}",
               termion::cursor::Goto(1, 1),
               termion::clear::CurrentLine)
                .unwrap();

        editor.process(c.unwrap());
        */
        /*
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
    */
}
