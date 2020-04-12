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
use clap::{Arg, App, crate_version};

use ballot::{Ballot, VoteError, VoteResult};
use console::Console;
use scanner::{Event, Scanner};

use util::{get_maplist, get_server_info};

fn main() {
    let matches = App::new("Democracy")
    .version(crate_version!())
    .author("Goatfoot")
    .about("A voting plugin for Movie Battles 2")
    .arg(Arg::with_name("maps")
        .short("m")
        .long("maps")
        .value_name("MAPS")
        .help("Sets the file that contains the map list")
        .default_value("./maps.txt"))
    .arg(Arg::with_name("rcon")
        .short("r")
        .long("rcon")
        .value_name("RCON")
        .help("Sets the rcon password")
        .default_value("password"))
    .arg(Arg::with_name("hostip")
        .short("i")
        .long("host-ip")
        .value_name("HOST IP")
        .help("Sets the host IP")
        .default_value("127.0.0.1"))
    .arg(Arg::with_name("hostport")
        .short("p")
        .long("host-port")
        .value_name("HOST PORT")
        .help("Sets the host port")
        .default_value("29070"))
    .arg(Arg::with_name("clientport")
        .short("c")
        .long("client-port")
        .value_name("CLIENT PORT")
        .help("Sets the client port")
        .default_value("3400"))
    .arg(Arg::with_name("log")
        .short("l")
        .long("log")
        .value_name("LOG")
        .help("Sets the game log file")
        .default_value("./games.log"))
    .arg(Arg::with_name("interval")
        .short("t")
        .long("interval")
        .value_name("INTERVAL")
        .help("Sets the update interval in second")
        .default_value("1"))
    .arg(Arg::with_name("timeout")
        .short("o")
        .long("timeout")
        .value_name("TIMEOUT")
        .help("Sets the timeout for server command in millisecond")
        .default_value("100"))
    .arg(Arg::with_name("votingduration")
        .short("d")
        .long("votingduration")
        .value_name("VOTING DURATION")
        .help("Sets the voting duration")
        .default_value("30"))
    .arg(Arg::with_name("playercooldown")
        .short("C")
        .long("playercooldown")
        .value_name("PLAYER COOLDOWN")
        .help("Sets the player cooldown")
        .default_value("30"))
    .arg(Arg::with_name("target")
        .short("x")
        .long("target")
        .value_name("TARGET")
        .help("Sets the voting target ratio")
        .default_value("0.6"))
    .get_matches();

    let maps = matches.value_of("maps").unwrap_or_default();
    let rcon = matches.value_of("rcon").unwrap_or_default();
    let hostip = matches.value_of("hostip").unwrap_or_default();
    let hostport: u16 = matches.value_of("hostport").unwrap_or_default().parse().expect("cannot read host port");
    let clientport: u16 = matches.value_of("clientport").unwrap_or_default().parse().expect("cannot read client port");
    let log = matches.value_of("log").unwrap_or_default();
    let interval: u64 = matches.value_of("interval").unwrap_or_default().parse().expect("cannot read interval");
    let timeout: u64 = matches.value_of("timeout").unwrap_or_default().parse().expect("cannot read timeour");
    let voting_duration: u64 = matches.value_of("votingduration").unwrap_or_default().parse().expect("cannot read interval");
    let player_cooldown: u64 = matches.value_of("playercooldown").unwrap_or_default().parse().expect("cannot read timeout");
    let target: f32 = matches.value_of("target").unwrap_or_default().parse().expect("cannot read target");

    let mut scanner = Scanner::new(log);
    let console = Console::new(
        rcon.to_owned(),
        hostip,
        hostport,
        clientport,
        Duration::from_millis(timeout),
    );
    let mut nominations = HashMap::new();
    nominations.insert(
        "map".to_owned(),
        get_maplist(maps).expect("can't find map list"),
    );
    let mut modes = HashSet::new();
    modes.insert("0".to_owned());
    modes.insert("1".to_owned());
    modes.insert("2".to_owned());
    modes.insert("3".to_owned());
    modes.insert("4".to_owned());
    nominations.insert("mode".to_owned(), modes);
    let ballot = Ballot::new(
        Duration::from_secs(voting_duration),
        Duration::from_secs(player_cooldown),
        target,
        nominations,
    );
    let mut system = System::new(console, ballot);
    loop {
        for event in scanner.events() {
            system.handle_event(event);
        }
        sleep(Duration::from_secs(interval));
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

    pub fn check_vote_result(&mut self, majority_result: bool) {
        match self.ballot.get_result(majority_result) {
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
                        self.check_vote_result(false);
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
                        self.check_vote_result(false);
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
