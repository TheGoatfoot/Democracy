use lazy_static::lazy_static;

use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

lazy_static! {
    static ref REGEX_INIT: Regex = Regex::new(r#"^ *(?P<minute>[0-9]+):(?P<second>[0-9]{2}) *InitGame:"#).unwrap();
    static ref REGEX_SHUTDOWN: Regex = Regex::new(r#"^ *(?P<minute>[0-9]+):(?P<second>[0-9]{2}) *ShutdownGame:"#).unwrap();
    static ref REGEX_CONNECT: Regex =
        Regex::new(r#"^ *(?P<minute>[0-9]+):(?P<second>[0-9]{2}) *ClientConnect: (?P<id>[0-9]{1,2})"#).unwrap();
    static ref REGEX_DISCONNECT: Regex =
        Regex::new(r#"^ *(?P<minute>[0-9]+):(?P<second>[0-9]{2}) *ClientDisconnect: (?P<id>[0-9]{1,2})"#).unwrap();
    static ref REGEX_CHAT: Regex = Regex::new(
        r#"^ *(?P<minute>[0-9]+):(?P<second>[0-9]{2}) *(?P<id>[0-9]{1,2}): say: (?P<username>.*): "(?P<message>.*)""#
    )
    .unwrap();
}

pub struct Scanner {
    buffer: BufReader<File>,
}

pub enum Event {
    Init(String, String),
    Shutdown(String, String),
    Connect(String, String, String),
    Disconnect(String, String, String),
    Chat(String, String, String, String, String),
}

impl Scanner {
    pub fn new(path: &str) -> Scanner {
        let file = File::open(path).expect("cannot open log file");
        let mut buffer: BufReader<File> = BufReader::new(file);
        buffer
            .seek(std::io::SeekFrom::End(0))
            .expect("buffer could not seek to end");
        Self { buffer: buffer }
    }

    pub fn events(&mut self) -> Events {
        Events {
            buffer: &mut self.buffer,
            string_buffer: String::new(),
        }
    }
}

pub struct Events<'a> {
    buffer: &'a mut BufReader<File>,
    string_buffer: String,
}

impl<'a> Iterator for Events<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Event> {
        self.string_buffer.clear();
        self.buffer
            .read_line(&mut self.string_buffer)
            .expect("error reading log file");
        let line = &self.string_buffer;
        match REGEX_INIT.captures(line) {
            Some(captures) => {
                let minute = captures
                    .name("minute")
                    .expect("could not get init 'minute'")
                    .as_str()
                    .to_owned();
                let second = captures
                    .name("second")
                    .expect("could not get init 'second'")
                    .as_str()
                    .to_owned();
                return Some(Event::Init(minute, second));
            }
            None => {}
        }
        match REGEX_SHUTDOWN.captures(line) {
            Some(captures) => {
                let minute = captures
                    .name("minute")
                    .expect("could not get shutdown 'minute'")
                    .as_str()
                    .to_owned();
                let second = captures
                    .name("second")
                    .expect("could not get shutdown 'second'")
                    .as_str()
                    .to_owned();
                return Some(Event::Shutdown(minute, second));
            }
            None => {}
        }
        match REGEX_CONNECT.captures(line) {
            Some(captures) => {
                let minute = captures
                    .name("minute")
                    .expect("could not get connect 'minute'")
                    .as_str()
                    .to_owned();
                let second = captures
                    .name("second")
                    .expect("could not get connect 'second'")
                    .as_str()
                    .to_owned();
                let id = captures
                    .name("id")
                    .expect("could not get connect 'id'")
                    .as_str()
                    .to_owned();
                return Some(Event::Connect(minute, second, id));
            }
            None => {}
        }
        match REGEX_DISCONNECT.captures(line) {
            Some(captures) => {
                let minute = captures
                    .name("minute")
                    .expect("could not get disconnect 'minute'")
                    .as_str()
                    .to_owned();
                let second = captures
                    .name("second")
                    .expect("could not get disconnect 'second'")
                    .as_str()
                    .to_owned();
                let id = captures
                    .name("id")
                    .expect("could not get disconnect 'id'")
                    .as_str()
                    .to_owned();
                return Some(Event::Disconnect(minute, second, id));
            }
            None => {}
        }
        match REGEX_CHAT.captures(line) {
            Some(captures) => {
                let minute = captures
                    .name("minute")
                    .expect("could not get chat 'minute'")
                    .as_str()
                    .to_owned();
                let second = captures
                    .name("second")
                    .expect("could not get chat 'second'")
                    .as_str()
                    .to_owned();
                let id = captures
                    .name("id")
                    .expect("could not get chat 'id'")
                    .as_str()
                    .to_owned();
                let username = captures
                    .name("username")
                    .expect("could not get chat 'username'")
                    .as_str()
                    .to_owned();
                let message = captures
                    .name("message")
                    .expect("could not get chat 'message'")
                    .as_str()
                    .to_owned();
                return Some(Event::Chat(minute, second, id, username, message));
            }
            None => {}
        }
        None
    }
}
