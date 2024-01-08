use std::fmt::Display;

use crate::notation::AlgebraicMove;

pub trait Move: Sized + Display {
    fn to_uci(&self) -> String;
}

pub trait ChessGame: Sized {
    type Move: Move;
    type UnMove;
    fn new() -> Self;
    fn from_fen(fen: &str) -> Option<Self>;
    fn decode_alg(&mut self, mov: &AlgebraicMove) -> Self::Move;
    fn gen_alg(&mut self, mov: &Self::Move) -> AlgebraicMove;
    fn moves(&mut self) -> Vec<Self::Move>;
    fn do_move(&mut self, mov: &Self::Move) -> Self::UnMove;
    fn unmove(&mut self, mov: &Self::UnMove);
}