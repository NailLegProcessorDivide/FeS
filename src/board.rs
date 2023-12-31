use std::fmt::{Display, Write};

use streaming_iterator::{StreamingIteratorMut, StreamingIterator};

use crate::{piece::{self, PlayerColor, Piece, ColouredPiece}, notation::AlgebraicMove};

pub struct FesMove {
    pub from: u8,
    /// top 2 bits = promotion type
    /// 00 queen
    /// 01 rook
    /// 10 bisharp
    /// 11 knight
    pub to: u8
}

struct FesMoveDet {
    from: u8,
    to: u8,
    promo: Option<Piece>,
    take: Option<(Piece, u8)>
}

pub trait ChessGame: Sized {
    type MovIter<'a>: StreamingIteratorMut<Item = Self> where Self: 'a;
    fn new() -> Self;
    fn from_fen(fen: &str) -> Option<Self>;
    fn move_alg(&mut self, mov: &AlgebraicMove);
    fn moves<'a>(&'a mut self) -> Self::MovIter<'a>;
}

pub struct Board {
    pieces: [[Option<piece::ColouredPiece>; 8]; 8],
}

impl Board {
    pub fn new() -> Self{
        Self {pieces: [[None; 8]; 8]}
    }
    pub fn from_fen(input: &str) -> Option<Self> {
        let mut board = Self {pieces: [[None; 8]; 8]};
        for (i, line) in input.split('/').enumerate() {
            if i >= 8 { return None; }
            let mut counter = 0;
            for c in line.chars() {
                if counter > 8 { return None; }
                if c.is_digit(10) {
                    counter += c as usize - '0' as usize;
                } else {
                    board.pieces[i][counter] = match c {
                        'P' => Some(piece::ColouredPiece::WhitePawn),
                        'N' => Some(piece::ColouredPiece::WhiteKnight),
                        'B' => Some(piece::ColouredPiece::WhiteBishop),
                        'R' => Some(piece::ColouredPiece::WhiteRook),
                        'Q' => Some(piece::ColouredPiece::WhiteQueen),
                        'K' => Some(piece::ColouredPiece::WhiteKing),
                        'p' => Some(piece::ColouredPiece::BlackPawn),
                        'n' => Some(piece::ColouredPiece::BlackKnight),
                        'b' => Some(piece::ColouredPiece::BlackBishop),
                        'r' => Some(piece::ColouredPiece::BlackRook),
                        'q' => Some(piece::ColouredPiece::BlackQueen),
                        'k' => Some(piece::ColouredPiece::BlackKing),
                        _ => return None
                    };
                    counter += 1;
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

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.pieces {
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
        let mut castle_rights = "".to_string();
        if self.white_ks_castle {castle_rights += "K"}
        if self.white_qs_castle {castle_rights += "Q"}
        if self.black_ks_castle {castle_rights += "k"}
        if self.black_qs_castle {castle_rights += "q"}
        f.write_str(&format!("Castle rights: {castle_rights}\n"))?;
        f.write_str(&format!("ep: {:?}\n", self.enpasont_col))?;
        f.write_str("Player: ")?;
        f.write_str( match self.turn {
            PlayerColor::White => "white",
            PlayerColor::Black => "black",
        })?;
        f.write_str(" to play.")
    }
}

pub struct BasicMoveGenerator<'a> {
    game_state: &'a mut GameState,
    mov: FesMoveDet,
    moves: Vec<FesMoveDet>
}

use ColouredPiece::*;

impl<'a> BasicMoveGenerator<'a> {
    fn from_gs(gs: &'a mut GameState) -> Self {
        let mut moves = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                match gs.board.pieces[y][x] {
                    Some(WhitePawn) => {
                        if x > 0 && ColouredPiece::opt_is_black(gs.board.pieces[y + 1][x - 1]) {
                            let p = match gs.board.pieces[y + 1][x - 1] {
                                Some(p) => p,
                                None => panic!("stuff imploded")
                            }.piece();
                            let from = y as u8 * 8 + x as u8;
                            let to = y as u8 * 8 + x as u8 + 7;
                            let take = Some((p, to));
                            if y == 6 {
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Queen), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Rook), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Knight), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Bishop), take });
                            }
                            else {
                                moves.push(FesMoveDet { from, to, promo: None, take });
                            }
                        }
                        if x < 7 && ColouredPiece::opt_is_black(gs.board.pieces[y + 1][x + 1]) {
                            let p = match gs.board.pieces[y + 1][x + 1] {
                                Some(p) => p,
                                None => panic!("stuff imploded")
                            }.piece();
                            let from = y as u8 * 8 + x as u8;
                            let to = y as u8 * 8 + x as u8 + 9;
                            let take = Some((p, to));
                            if y == 6 {
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Queen), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Rook), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Knight), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Bishop), take });
                            }
                            else {
                                moves.push(FesMoveDet { from, to, promo: None, take });
                            }
                        }
                        if gs.board.pieces[y + 1][x].is_none() {
                            let from = y as u8 * 8 + x as u8;
                            let to = y as u8 * 8 + x as u8 + 8;
                            if y == 6 {
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Queen), take: None });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Rook), take: None });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Knight), take: None });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Bishop), take: None });
                            }
                            else {
                                moves.push(FesMoveDet { from, to, promo: None, take: None });
                            }
                            if y == 1 && gs.board.pieces[y + 2][x].is_none() {
                                let from = y as u8 * 8 + x as u8;
                                let to = y as u8 * 8 + x as u8 + 16;
                                moves.push(FesMoveDet { from, to, promo: None, take: None });
                            }
                        }
                    },
                    Some(WhiteKnight) => {

                    },
                    Some(WhiteBishop) => todo!(),
                    Some(WhiteRook) => todo!(),
                    Some(WhiteQueen) => todo!(),
                    Some(WhiteKing) => todo!(),
                    Some(BlackPawn) => {
                        if x > 0 && ColouredPiece::opt_is_white(gs.board.pieces[y - 1][x - 1]) {
                            let p = match gs.board.pieces[y - 1][x - 1] {
                                Some(p) => p,
                                None => panic!("stuff imploded")
                            }.piece();
                            let from = y as u8 * 8 + x as u8;
                            let to = y as u8 * 8 + x as u8 - 9;
                            let take = Some((p, to));
                            if y == 6 {
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Queen), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Rook), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Knight), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Bishop), take });
                            }
                            else {
                                moves.push(FesMoveDet { from, to, promo: None, take });
                            }
                        }
                        if x < 7 && ColouredPiece::opt_is_white(gs.board.pieces[y - 1][x + 1]) {
                            let p = match gs.board.pieces[y - 1][x + 1] {
                                Some(p) => p,
                                None => panic!("stuff imploded")
                            }.piece();
                            let from = y as u8 * 8 + x as u8;
                            let to = y as u8 * 8 + x as u8 - 7;
                            let take = Some((p, to));
                            if y == 6 {
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Queen), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Rook), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Knight), take });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Bishop), take });
                            }
                            else {
                                moves.push(FesMoveDet { from, to, promo: None, take });
                            }
                        }
                        if gs.board.pieces[y - 1][x].is_none() {
                            let from = y as u8 * 8 + x as u8;
                            let to = y as u8 * 8 + x as u8 - 8;
                            if y == 1 {
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Queen), take: None });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Rook), take: None });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Knight), take: None });
                                moves.push(FesMoveDet { from, to, promo: Some(Piece::Bishop), take: None });
                            }
                            else {
                                moves.push(FesMoveDet { from, to, promo: None, take: None });
                            }
                            if y == 6 && gs.board.pieces[y - 2][x].is_none() {
                                let from = y as u8 * 8 + x as u8;
                                let to = y as u8 * 8 + x as u8 - 16;
                                moves.push(FesMoveDet { from, to, promo: None, take: None });
                            }
                        }
                    },
                    Some(BlackKnight) => todo!(),
                    Some(BlackBishop) => todo!(),
                    Some(BlackRook) => todo!(),
                    Some(BlackQueen) => todo!(),
                    Some(BlackKing) => todo!(),
                    None => todo!(),
                }
            }
        }

        Self { game_state: gs, mov: FesMoveDet { from: 0, to: 0, promo: None, take: None }, moves }
    }

    /// if a move has been made, undo it
    /// else, nop
    fn unmove(&mut self) {
        if self.mov.from != 0 || self.mov.to != 0 {
            todo!()
        }
        self.mov = FesMoveDet { from: 0, to: 0, promo: None, take: None };
    }
}

impl<'a> StreamingIterator for BasicMoveGenerator<'a> {
    type Item = GameState;

    fn advance(&mut self) {
        self.unmove();
    }

    fn get(&self) -> Option<&Self::Item> {
        todo!()
    }
}

impl<'a> StreamingIteratorMut for BasicMoveGenerator<'a> {
    fn get_mut(&mut self) -> Option<&mut Self::Item> {
        todo!()
    }

    fn next_mut(&mut self) -> Option<&mut Self::Item> {
        self.advance();
        (*self).get_mut()
    }
}

impl<'a> Drop for BasicMoveGenerator<'a> {
    fn drop(&mut self) {
        self.unmove()
    }
}

impl ChessGame for GameState {
    type MovIter<'a> = BasicMoveGenerator<'a>;

    fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    fn from_fen(input: &str) -> Option<Self> {
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

    fn move_alg(&mut self, mov: &AlgebraicMove) {
        todo!()
    }

    fn moves<'a>(&'a mut self) -> Self::MovIter<'a> {
        BasicMoveGenerator::from_gs(self)
    }
}
