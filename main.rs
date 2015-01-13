#![crate_name = "rush"]
#![allow(dead_code)]
#![feature(slicing_syntax)]
#![feature(box_syntax)]
#![allow(unstable)]

use std::path::Path;
use std::io::{BufferedReader, File, IoResult};

enum Mode { Bourne, BourneAgain }

struct Config {
    mode: Mode,
    input_file: Option<String>,
}

struct Session {
    reader: Box<InputReader + 'static>
}

trait InputReader {
    fn read_input(&mut self) -> IoResult<Option<String>>;
}

struct FileInputReader {
    reader: BufferedReader<File>
}

impl InputReader for FileInputReader {
    fn read_input(&mut self) -> IoResult<Option<String>> {
        use std::io::{IoError, EndOfFile};

        match self.reader.read_line() {
            Ok(s) => Ok(Some(s)),
            Err(IoError { kind: EndOfFile, .. }) => Ok(None),
            Err(e) => Err(e)
        }
    }
}

struct ConsoleInputReader;

impl InputReader for ConsoleInputReader {
    fn read_input(&mut self) -> IoResult<Option<String>> {
        unimplemented!()
    }
}

struct Parser;

impl Parser {
    fn new() -> Parser { Parser }

    fn parse_words(&mut self, line: String) -> IoResult<Vec<String>> {
        let mut words: Vec<String> = Vec::new();
        let mut word: Vec<char> = Vec::new();
        for c in line.chars() {
            if c == ' ' || c == '\n' {
                if !word.is_empty() {
                    words.push(word.clone().into_iter().collect());
                    word.clear();
                }
            } else {
                word.push(c);
            }
        }

        if !word.is_empty() {
            words.push(word.clone().into_iter().collect());
            word.clear();
        }

        return Ok(words);
    }
}

fn main() {
    let config = parse_config_from_args();
    let ref mut session = match create_session(config) {
        Ok(s) => s,
        Err(e) => {
            report_error(&e.to_string()[]);
            std::os::set_exit_status(1);
            return
        }
    };
    let ref mut parser = Parser::new();

    loop {
        match interpret_next_line(session, parser) {
            Ok(Some(..)) => (),
            Ok(None) => return,
            Err(e) => {
                report_error(&e.to_string()[]);
                std::os::set_exit_status(1);
                return
            }
        }
    }
}

fn parse_config_from_args() -> Config {
    let args = std::os::args();

    if args.len() > 1 {
        Config {
            mode: Mode::Bourne,
            input_file: Some(args[1].clone())
        }
    } else {
        unimplemented!()
    }
}

fn create_session(config: Config) -> IoResult<Session> {
    let reader = match config.input_file {
        Some(f) => {
            let ref path = Path::new(f);
            let file = try!(File::open(path));
            let reader = BufferedReader::new(file);
            let reader = FileInputReader { reader: reader };
            box reader
        }
        None => unimplemented!()
    };

    Ok(Session {
        reader: reader
    })
}

fn report_error(s: &str) {
    let _ = writeln!(&mut std::io::stderr(), "rush: {}", s);
}

fn execute_command(words: &[String]) {
    use std::io::Command;
    use std::io::process::StdioContainer::InheritFd;

    if words.is_empty() { return }

    let name = &words[..1];
    let args = &words[1..];

    let mut cmd = Command::new(&name[0][]);
    cmd.args(args);
    match cmd.stdin(InheritFd(0)).stdout(InheritFd(1)).stderr(InheritFd(1)).spawn() {
        Ok(mut p) => {
            let _ = p.wait();
        }
        Err(e) => println!("{}", e)
    }
}

fn interpret_next_line(session: &mut Session, parser: &mut Parser) -> IoResult<Option<()>> {
    let line = try!(session.reader.read_input());
    match line {
        Some(line) => {
            let words = try!(parser.parse_words(line));
            execute_command(&words[]);
            Ok(Some(()))
        }
        None => Ok(None)
    }
}
