use std::fmt::Display;

use crate::notation::AlgebraicMove;

pub trait Move: Sized + Display {
    fn to_uci(&self) -> String;
}

impl Move for u16 {
    fn to_uci(&self) -> String {
        let ox = ('h' as u8 - (self & 7) as u8) as char;
        let oy = ('1' as u8 + ((self >> 3) & 7) as u8) as char;
        let nx = ('h' as u8 - ((self >> 6) & 7) as u8) as char;
        let ny = ('1' as u8 + ((self >> 9) & 7) as u8) as char;
        format!("{ox}{oy}{nx}{ny}")
    }
}

pub trait ChessGame: Sized {
    type Move: Move;
    type UnMove;
    fn new() -> Self;
    fn from_fen(fen: &str) -> Option<Self>;
    fn decode_alg(&mut self, mov: &AlgebraicMove) -> Self::Move;
    fn gen_alg(&mut self, mov: &Self::Move) -> AlgebraicMove;
    fn moves(&self) -> Vec<Self::Move>;
    fn do_move(&mut self, mov: &Self::Move) -> Self::UnMove;
    fn unmove(&mut self, mov: &Self::UnMove);
}
