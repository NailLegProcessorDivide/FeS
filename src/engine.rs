use lazy_static::lazy_static;
use regex::Regex;
use std::io::{self, BufRead};

pub struct GoArgs<'a> {
    pub moves: Option<Vec<&'a str>>,
    pub ponder: bool,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u64>,
    pub depth: Option<u64>,
    pub nodes: Option<u64>,
    pub mate: Option<u64>,
    pub move_time: Option<u64>,
    pub infinite: bool,
}

pub trait Engine {
    fn new() -> Self;
    fn set_from_fen(&mut self, fen: &str);
    fn play_move(&mut self, mov: &str);
    fn select_move(&self) -> u16;
    fn get_name(&self) -> String;
    fn get_author(&self) -> String;
    fn set_debug(&self, b: bool);
    fn log(&self, log: &str);
    fn go<'a>(&self, args: &'a GoArgs);
    fn stop(&self);
}

lazy_static! {
    static ref START_POS: Regex = Regex::new(r"^ ?startpos( moves(( [a-h][1-8][a-h][1-8][rnbq]?)+))?").unwrap();
    static ref FEN_POS: Regex = Regex::new(r"^ ?([pnbrqkPNBRQK1-8/]* [bw] ((K?Q?k?q?)|\-) (([a-h][1-8])|\-)( (([1-9][0-9]*)|\-|0))?( (([1-9][0-9]*)|\-|0))?)( moves(( [a-h][1-8][a-h][1-8][rnbq]?)+))?").unwrap();
    static ref SEARCH_MOVES: Regex = Regex::new(r"searchmoves(( [a-h][1-8][a-h][1-8][rnbq]?)+)").unwrap();
    static ref PONDER: Regex = Regex::new(r"ponder").unwrap();
    static ref WTIME: Regex = Regex::new(r"wtime (([1-9][0-9]*)|0)").unwrap();
    static ref BTIME: Regex = Regex::new(r"btime (([1-9][0-9]*)|0)").unwrap();
    static ref WINC: Regex = Regex::new(r"winc (([1-9][0-9]*)|0)").unwrap();
    static ref BINC: Regex = Regex::new(r"binc (([1-9][0-9]*)|0)").unwrap();
    static ref MOVES_TO_GO: Regex = Regex::new(r"movestogo (([1-9][0-9]*)|0)").unwrap();
    static ref DEPTH: Regex = Regex::new(r"depth (([1-9][0-9]*)|0)").unwrap();
    static ref NODES: Regex = Regex::new(r"nodes (([1-9][0-9]*)|0)").unwrap();
    static ref MATE: Regex = Regex::new(r"mate (([1-9][0-9]*)|0)").unwrap();
    static ref MOVE_TIME: Regex = Regex::new(r"movetime (([1-9][0-9]*)|0)").unwrap();
    static ref INFINITE: Regex = Regex::new(r"infinite").unwrap();
}

pub fn do_uci<Eng: Engine>(eng: &mut Eng) {
    let stdin = io::stdin();
    let mut iterator = stdin.lock().lines();

    loop {
        let input = iterator.next().unwrap().unwrap() + " ";
        let command = input.trim().split_once(" ");
        match command {
            Some(("uci", _)) => {
                println!("id name {}", eng.get_name());
                println!("id author {}", eng.get_author());
                println!("uciok");
            }
            Some(("debug", "on")) => {
                eng.set_debug(true);
            }
            Some(("debug", "off")) => {
                eng.set_debug(false);
            }
            Some(("isready", _)) => {
                println!("readyok");
            }
            Some(("setoption", rest)) => {
                eng.log(&format!("tried to set option {rest}"));
            }
            Some(("register", rest)) => {
                todo!("tried to register {rest}");
            }
            Some(("ucinewgame", rest)) => {
                eng.log(&format!("starting new game: {rest}"));
            }
            Some(("position", rest)) => {
                let sp = match START_POS.captures(rest) {
                    Some(m) => m.get(2),
                    None => match FEN_POS.captures(rest) {
                        Some(m) => m.get(13),
                        None => {
                            eng.log(&format!("sp no match {rest}"));
                            continue;
                        }
                    },
                };
                match sp {
                    Some(moves) => {
                        moves
                            .as_str()
                            .trim()
                            .split_whitespace()
                            .for_each(|m| eng.play_move(m));
                    }
                    None => eng.log("no moves"),
                }
            }
            Some(("go", rest)) => {
                let moves = SEARCH_MOVES.captures(rest).map(|m| {
                    m.get(1)
                        .unwrap()
                        .as_str()
                        .trim()
                        .split_whitespace()
                        .collect()
                });
                let ponder = PONDER.is_match(rest);
                let wtime = WTIME
                    .captures(rest)
                    .and_then(|m| m.get(1).unwrap().as_str().parse::<u64>().ok());
                let btime = match BTIME.captures(rest) {
                    Some(m) => m.get(1).unwrap().as_str().parse::<u64>().ok(),
                    None => None,
                };
                let winc = match WINC.captures(rest) {
                    Some(m) => m.get(1).unwrap().as_str().parse::<u64>().ok(),
                    None => None,
                };
                let binc = match BINC.captures(rest) {
                    Some(m) => m.get(1).unwrap().as_str().parse::<u64>().ok(),
                    None => None,
                };
                let movestogo = match MOVES_TO_GO.captures(rest) {
                    Some(m) => m.get(1).unwrap().as_str().parse::<u64>().ok(),
                    None => None,
                };
                let depth = match DEPTH.captures(rest) {
                    Some(m) => m.get(1).unwrap().as_str().parse::<u64>().ok(),
                    None => None,
                };
                let nodes = match NODES.captures(rest) {
                    Some(m) => m.get(1).unwrap().as_str().parse::<u64>().ok(),
                    None => None,
                };
                let mate = match MATE.captures(rest) {
                    Some(m) => m.get(1).unwrap().as_str().parse::<u64>().ok(),
                    None => None,
                };
                let move_time = match MOVE_TIME.captures(rest) {
                    Some(m) => m.get(1).unwrap().as_str().parse::<u64>().ok(),
                    None => None,
                };
                let infinite = INFINITE.is_match(rest);
                let garg = GoArgs {
                    moves,
                    ponder,
                    wtime,
                    btime,
                    winc,
                    binc,
                    movestogo,
                    depth,
                    nodes,
                    mate,
                    move_time,
                    infinite,
                };
                eng.go(&garg);
            }
            Some(("stop", _)) => eng.stop(),
            Some(("quit", _)) => {
                eng.stop();
                return;
            }
            Some((t, _)) => {
                eng.log(&format!("unknown command \"{t}\""));
            }
            None => todo!(),
        }
    }
}
