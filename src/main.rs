mod ballot;
mod console;
mod cooldown;
mod scanner;
mod util;

use lazy_static::lazy_static;

use std::collections::{HashMap, HashSet};
use std::thread::sleep;
use std::time::Duration;

use regex::Regex;

use ballot::{Ballot, VoteError, VoteResult};
use console::Console;
use scanner::{Event, Scanner};

use util::{get_maplist, get_server_info};

fn main() {
    let mut scanner = Scanner::new("C:/Users/Rania/Documents/My Games/OpenJK/MBII/games.log");
    let console = Console::new(
        "heererer".to_owned(),
        "127.0.0.1",
        29070,
        3400,
        Duration::from_millis(100),
    );
    let mut nominations = HashMap::new();
    nominations.insert(
        "map".to_owned(),
        get_maplist("./maps.txt").expect("can't find map list"),
    );
    let mut modes = HashSet::new();
    modes.insert("0".to_owned());
    modes.insert("1".to_owned());
    modes.insert("2".to_owned());
    modes.insert("3".to_owned());
    modes.insert("4".to_owned());
    nominations.insert("mode".to_owned(), modes);
    let ballot = Ballot::new(
        Duration::from_secs(30),
        Duration::from_secs(30),
        0.6,
        nominations,
    );
    let mut system = System::new(console, ballot);
    loop {
        for event in scanner.events() {
            system.handle_event(event);
        }
        system.check_vote_result(false, false);
        sleep(Duration::from_secs(1));
    }
}

lazy_static! {
    static ref REGEX_CHAT_PROPOSE: Regex =
        Regex::new(r#"^vote (?P<type>map|mode) (?P<input>.*)"#).unwrap();
    static ref REGEX_CHAT_VOTE: Regex = Regex::new(r#"^(?P<vote>yay|nay)"#).unwrap();
}

pub struct System {
    console: Console,
    ballot: Ballot,
}

impl System {
    pub fn new(console: Console, ballot: Ballot) -> System {
        System {
            console: console,
            ballot: ballot,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Init(minute, second) => {}
            Event::Shutdown(minute, second) => {}
            Event::Connect(minute, second, id) => {
                self.ballot.increment_voters();
            }
            Event::Disconnect(minute, second, id) => {
                self.ballot.unvote(&id);
                self.ballot.decrement_voters();
            }
            Event::Chat(minute, second, id, username, message) => {
                self.handle_message(&id, &username, &message);
            }
        }
    }

    pub fn check_vote_result(&mut self, ignore_cooldown: bool, majority_result: bool) {
        match self.ballot.get_result(ignore_cooldown, majority_result) {
            Ok(result) => {
                match result {
                    VoteResult::Yay(r#type, input) => {
                        self.console.svsay(b"Yay vote majority, motion granted.");
                        match r#type.as_str() {
                            "map" => {
                                self.console.map(input.as_bytes());
                            }
                            "mode" => {
                                self.console.mbmode(input.as_bytes());
                            }
                            _ => {}
                        }
                    }
                    VoteResult::Nay => {
                        self.console.svsay(b"Nay vote majority, motion denied.");
                    }
                    VoteResult::None => {
                        self.console.svsay(b"Voting deadlock, motion denied.");
                    }
                }
                self.ballot.stop_voting();
            }
            Err(error) => match error {
                VoteError::Voters => {
                    self.ballot.stop_voting();
                }
                _ => {}
            },
        }
    }

    fn handle_message(&mut self, id: &String, username: &String, message: &String) {
        match REGEX_CHAT_PROPOSE.captures(message) {
            Some(captures) => {
                let r#type = captures
                    .name("type")
                    .expect("could not get propose 'type'")
                    .as_str()
                    .to_owned();
                let input = captures
                    .name("input")
                    .expect("could not get propose 'type'")
                    .as_str()
                    .to_owned();
                match self.ballot.start_voting(id, &r#type, &input) {
                    Ok(_) => {
                        self.ballot
                            .set_voters(get_server_info(&mut self.console).unwrap());
                        self.console
                            .svsay(
                                &[
                                    self.ballot.get_type().as_bytes(),
                                    b" '",
                                    input.as_bytes(),
                                    b"' is nominated!",
                                ]
                                .concat(),
                            )
                            .unwrap();
                        self.console.svsay(b"type 'yay' to vote yes").unwrap();
                        self.console.svsay(b"type 'nay' to vote no").unwrap();
                        let (yay, nay) = self.ballot.get_requirements();
                        self.console
                            .svsay(format!("{} yay vote(s) needed for motion", yay).as_bytes())
                            .unwrap();
                        self.console
                            .svsay(format!("{} nay vote(s) needed to deny", nay).as_bytes())
                            .unwrap();
                        self.ballot.vote(id, true);
                        self.check_vote_result(true, true);
                    }
                    Err(error) => match error {
                        VoteError::Cooldown(duration) => {
                            self.console
                                .svsay(format!("User '{}' is in cooldown for {:.2} second!", username, duration).as_bytes())
                                .unwrap();
                        }
                        VoteError::Progress => {
                            self.console
                                .svsay(b"Voting is currently in progress!")
                                .unwrap();
                        }
                        VoteError::Nomination => {
                            self.console
                                .svsay(format!("Map '{}' is not on the list!", input).as_bytes())
                                .unwrap();
                        }
                        _ => {}
                    },
                }
            }
            None => {}
        }
        match REGEX_CHAT_VOTE.captures(message) {
            Some(captures) => {
                let vote = captures
                    .name("vote")
                    .expect("could not get propose 'type'")
                    .as_str();
                match match vote {
                    "yay" => self.ballot.vote(id, true),
                    "nay" => self.ballot.vote(id, false),
                    _ => Err(VoteError::Progress),
                } {
                    Ok(_) => {
                        self.print_requirements();
                        self.check_vote_result(true, true);
                    }
                    Err(_) => {}
                };
            }
            None => {}
        }
    }

    fn print_requirements(&mut self) {
        let (yay, nay) = self.ballot.get_votes();
        let (yay_needed, nay_needed) = self.ballot.get_requirements();
        self.console
            .svsay(format!("{}/{} yay - {}/{} nay", yay, yay_needed, nay, nay_needed).as_bytes());
    }
}
