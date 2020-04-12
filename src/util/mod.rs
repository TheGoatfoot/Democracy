use lazy_static::lazy_static;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Result};
use std::collections::HashSet;

use regex::Regex;

use crate::console::Console;

lazy_static! {
    static ref REGEX_INFO: Regex =
        Regex::new(r#"\\g_humanplayers\\(?P<player_count>[0-9]{1,2})\\"#).unwrap();
}

pub fn get_maplist(path: &str) -> Result<HashSet<String>> {
    let file = File::open(path)?;
    let file = BufReader::new(file);
    let mut output: HashSet<String> = HashSet::new();
    for line in file.lines() {
        output.insert(line?);
    }
    Ok(output)
}

pub fn get_server_info(console: &mut Console) -> Option<i8> {
    for message in console.send(b"getinfo").expect("could not get server info") {
        match REGEX_INFO.captures(&message) {
            Some(captures) => {
                return Some(
                    captures
                        .name("player_count")
                        .expect("could not get player count")
                        .as_str()
                        .parse()
                        .expect("could not parse player count"),
                );
            }
            None => {}
        }
    }
    None
}
