use std::fmt::{Display, Write};

use crate::piece::{self, PlayerColor};

pub trait ChessGame {
    fn new() -> Self;
    fn from_fen(fen: &str) -> Self;
    fn move_alg(&mut self, mov: &str);
}

pub struct Board {
    peices: [[Option<piece::ColouredPiece>; 8]; 8],
}

impl Board {
    pub fn new() -> Self{
        Self {peices: [[None; 8]; 8]}
    }
    pub fn from_fen(input: &str) -> Option<Self>{
        let mut board = Self {peices: [[None; 8]; 8]};
        for (i, line) in input.split('/').enumerate() {
            if i >= 8 {return None};
            let mut counter = 0;
            for c in line.chars() {
                if counter > 8 {return None}
                match c {
                    'P' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::WhitePawn);
                        counter += 1;
                    }
                    'N' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::WhiteKnight);
                        counter += 1;
                    }
                    'B' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::WhiteBishop);
                        counter += 1;
                    }
                    'R' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::WhiteRook);
                        counter += 1;
                    }
                    'Q' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::WhiteQueen);
                        counter += 1;
                    }
                    'K' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::WhiteKing);
                        counter += 1;
                    }
                    'p' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::BlackPawn);
                        counter += 1;
                    }
                    'n' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::BlackKnight);
                        counter += 1;
                    }
                    'b' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::BlackBishop);
                        counter += 1;
                    }
                    'r' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::BlackRook);
                        counter += 1;
                    }
                    'q' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::BlackQueen);
                        counter += 1;
                    }
                    'k' => {
                        board.peices[i][counter] = Some(piece::ColouredPiece::BlackKing);
                        counter += 1;
                    }
                    n if n.is_digit(10) => counter += n as usize - '0' as usize,
                    _ => return None
                }
            }
        }
        Some(board)
    }
}

pub struct GameState {
    turn: piece::PlayerColor,
    board: Board,
    /// White kingside castle
    white_ks_castle: bool,
    /// Black kingside castle
    black_ks_castle: bool,
    /// White queenside castle
    white_qs_castle: bool,
    /// Black queenside castle
    black_qs_castle: bool,
    enpasont_col: Option<u8>,
}

impl GameState {
    pub fn from_fen(input: &str) -> Option<Self> {
        let mut input_parts = input.trim().split(" ");
        let board = Board::from_fen(input_parts.next()?)?;
        let turn = match input_parts.next()? {
            "w" => PlayerColor::White,
            "b" => PlayerColor::Black,
            _ => return None
        };
        
        let castle_rights = input_parts.next()?;
        let white_ks_castle = castle_rights.contains('K');
        let white_qs_castle = castle_rights.contains('Q');
        let black_ks_castle = castle_rights.contains('k');
        let black_qs_castle = castle_rights.contains('q');

        let enpasont_col = match input_parts.next()?.chars().next()? {
            'a' => Some(0),
            'b' => Some(1),
            'c' => Some(2),
            'd' => Some(3),
            'e' => Some(4),
            'f' => Some(5),
            'g' => Some(6),
            'h' => Some(7),
            _ => None,
        };
        println!("e");

        Some(GameState { turn, board, white_ks_castle, white_qs_castle, black_ks_castle, black_qs_castle, enpasont_col })
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.peices {
            for piece in row {
                if let Some(p) = piece {
                    p.fmt(f)?;
                }
                else {
                    f.write_char('.')?;
                }
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.board.fmt(f)?;
        f.write_str("Player: ")?;
        f.write_str( match self.turn {
            PlayerColor::White => "white",
            PlayerColor::Black => "black",
        })?;
        f.write_str(" to play.\n")
    }
}
