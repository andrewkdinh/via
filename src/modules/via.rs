extern crate termion;
extern crate regex;

// use termion::event::Key;

use regex::Regex;

use super::editor::Editor;
use super::piece_table::PieceTable;

#[derive(Debug)]
/// Via main class, comprised of `Editor`'s
pub struct Via {
    /// List of current editors
    editors: Vec<Editor>,
    /// List of files to edit
    file_paths: Vec<String>,
    /// Configured options
    options: ViaOptions,
    /// Piece table of the command line
    cmd_piece_table: PieceTable,
}

impl Via {
    /// Initialize a new instance of Via from arguments
    pub fn new(args: Vec<String>) -> Via {
        let (mut file_paths, options) = process_args(args);
        if file_paths.is_empty() {
            file_paths.push("".to_string());
        }
        Via {
            editors: vec![Editor::new((*file_paths.first().unwrap()).as_str().to_string())],
            file_paths: file_paths,
            options: options,
            cmd_piece_table: PieceTable::new(),
        }
    }

    /*
    fn initialize(editor_options: editor::ViaOptions) -> editor::Editor {
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
        let eof_reached = read_lines(&mut reader, 1000000000000000, &mut initial_text); // TODO: FIX ME
        // TODO: Add this to initialization of piece table

        editor::Editor::new(piece_table::PieceTable::new(initial_text), reader, eof_reached, editor_options)
    }
    */
}

/// Process command line options and returns the files to edit and ViaOptions
// fn process_args(&mut self, args: &Vec<String>) -> Result<(Vec<String>, ViaOptions), &str> {
    fn process_args(args: Vec<String>) -> (Vec<String>, ViaOptions) {
        let mut flags = Vec::new();
        let mut file_paths = Vec::new();
        let flags_regex = Regex::new(r"--?\w+").unwrap();
        // let default_files = vec!["target/debug/via", "via"];
        for arg in &args[1..] {
            if flags_regex.is_match(&arg) {
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

#[derive(Debug)]
struct ViaOptions {
    /// Level of verboseness
    verboseness: usize,
}

impl ViaOptions {
    /// Return default options
    pub(crate) fn new() -> ViaOptions {
        ViaOptions {verboseness: 1}
    }
}