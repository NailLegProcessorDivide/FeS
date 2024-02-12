use std::{io::{self, BufRead}, time::Instant};

use fes::{
    bit_board::BitBoardGame, game::{ChessGame, Move}, perft_bb_mover::PerftMove
};

pub fn perft<Game: ChessGame>(gs: &mut Game, limit: usize) -> usize {
    if limit == 0 {
        1
    } else if limit == 1 {
        gs.moves().len()
    } else {
        let moves = gs.moves();
        let mut total = 0;
        for mov in moves {
            let unmov = gs.do_move(&mov);
            total += perft(gs, limit - 1);
            gs.unmove(&unmov);
        }
        total
    }
}

pub fn perft_div<Game: ChessGame>(gs: &mut Game, limit: usize) -> usize {
    let mut total = 0;
    for mov in gs.moves().iter() {
        let unmov = gs.do_move(mov);
        let c = perft(gs, limit - 1);
        total += c;
        println!("{}: {}", mov.to_uci(), c);
        gs.unmove(&unmov);
    }
    println!("\ntotal: {total}\n");
    total
}

fn main() {
    let stdin = io::stdin();
    let mut iterator = stdin.lock().lines();

    let mut gs = BitBoardGame::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    loop {
        let input = iterator.next().unwrap().unwrap();
        let mut parts = input.trim().split_whitespace();

        match parts.next() {
            Some(x) => match x {
                        "fen" => {
                            let fen = input[3..].trim();
                            gs = BitBoardGame::from_fen(&fen).unwrap();
                        }
                        "move" => {
                            'uci: for ucimov in parts {
                                for mov in gs.moves().iter() {
                                    if ucimov == mov.to_uci() {
                                        gs.do_move(mov);
                                        continue 'uci;
                                    }
                                }
                                println!("warn: unknown uci move {ucimov}! stopping");
                                break;
                            }
                        }
                        "perft" => {
                            let now = Instant::now();
                            let depth = parts.next().unwrap().parse::<usize>().unwrap();
                            perft_div(&mut gs, depth);
                            println!("{}ms", now.elapsed().as_millis());
                        }
                        "perft2" => {
                            let now = Instant::now();
                            let depth = parts.next().unwrap().parse::<u64>().unwrap();
                            let mut cont = PerftMove{ depth_target: depth, depth: 0, counter: 0 };
                            gs.proc_movs(&mut cont);
                            println!("total: {}", cont.counter);
                            println!("{}ms", now.elapsed().as_millis());
                        }
                        "quit" => { break; }
                        _ => { println!("Unrecognised command.") }
                    },
            None => {}
        }
    }
}
