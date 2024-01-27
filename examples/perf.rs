use std::io::{self, BufRead};

use fes::{
    bit_board::{BitBoardGame},
    game::{ChessGame, Move},
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
    let fen = iterator.next().unwrap().unwrap();
    let depthstr = iterator.next().unwrap().unwrap();
 
    let depth = depthstr.parse::<usize>().unwrap();

    let mut gs = BitBoardGame::from_fen(&fen).unwrap();

    perft_div(&mut gs, depth);
}
