use std::fmt::{Display, Write};

use crate::{piece::{self, PlayerColour, Piece, ColouredPiece}, notation::AlgebraicMove};

pub trait ChessGame: Sized {
    fn new() -> Self;
    fn from_fen(fen: &str) -> Option<Self>;
    fn move_alg(&mut self, mov: &AlgebraicMove);
    fn moves(&mut self) -> Vec<FesMoveDet>;
    fn move_det(&mut self, mov: &FesMoveDet);
    fn unmove_det(&mut self, mov: &FesMoveDet);
}


pub struct FesMove {
    pub from: u8,
    /// top 2 bits = promotion type
    /// 00 queen
    /// 01 rook
    /// 10 bisharp
    /// 11 knight
    pub to: u8
}

#[derive(Clone, Debug)]
struct GSMetaData {
    /// White kingside castle
    white_ks_castle: bool,
    /// Black kingside castle
    black_ks_castle: bool,
    /// White queenside castle
    white_qs_castle: bool,
    /// Black queenside castle
    black_qs_castle: bool,
    enpasant_col: Option<u8>,
}

/// Fes Move Detailed
/// move containing unpacked promotion and taking info
#[derive(Debug)]
pub struct FesMoveDet {
    pub from: u8,
    pub to: u8,
    promo: Option<Piece>,
    take: Option<Piece>,
    enpas: bool,
    meta: GSMetaData,
}
impl FesMoveDet {
    fn push_basic(vec: &mut Vec<FesMoveDet>, from: usize, to: usize, meta: &GSMetaData) {
        vec.push(FesMoveDet { from: from as u8, to: to as u8, promo: None, take: None, enpas: false, meta: meta.clone() })
    }
    fn push_take(vec: &mut Vec<FesMoveDet>, from: usize, to: usize, take: Option<Piece>,  meta: &GSMetaData) {
        vec.push(FesMoveDet { from: from as u8, to: to as u8, promo: None, take, enpas: false, meta: meta.clone() })
    }
    fn push_promo(vec: &mut Vec<FesMoveDet>, from: usize, to: usize, promo: Piece, take: Option<Piece>,  meta: &GSMetaData) {
        vec.push(FesMoveDet { from: from as u8, to: to as u8, promo: Some(promo), take, enpas: false, meta: meta.clone() })
    }
    fn push_enpas(vec: &mut Vec<FesMoveDet>, from: usize, to: usize, meta: &GSMetaData) {
        //takes none because the square it goes to isnt a piece (weird design IK)
        vec.push(FesMoveDet { from: from as u8, to: to as u8, promo: None, take: None, enpas: true, meta: meta.clone() })
    }
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
                    board.pieces[7 - i][counter] = match c {
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
    turn: piece::PlayerColour,
    board: Board,
    meta: GSMetaData,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.pieces.iter().rev() {
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
        if self.meta.white_ks_castle {castle_rights += "K"}
        if self.meta.white_qs_castle {castle_rights += "Q"}
        if self.meta.black_ks_castle {castle_rights += "k"}
        if self.meta.black_qs_castle {castle_rights += "q"}
        f.write_str(&format!("Castle rights: {castle_rights}\n"))?;
        f.write_str(&format!("ep: {:?}\n", self.meta.enpasant_col))?;
        f.write_str("Player: ")?;
        f.write_str( match self.turn {
            PlayerColour::White => "white",
            PlayerColour::Black => "black",
        })?;
        f.write_str(" to play.")
    }
}

use ColouredPiece::*;
use PlayerColour::*;

const fn legal_pos(x: usize, y: usize) -> bool {
    x < 8 && y < 8
}

const fn unpack_index(packed: u8) -> (usize, usize) {
    (packed as usize & 7, packed as usize >> 3)
}

const fn pack(x: usize, y: usize) -> usize {
    y * 8 + x
}

impl ChessGame for GameState {

    fn new() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    fn from_fen(input: &str) -> Option<Self> {
        let mut input_parts = input.trim().split(" ");
        let board = Board::from_fen(input_parts.next()?)?;
        let turn = match input_parts.next()? {
            "w" => PlayerColour::White,
            "b" => PlayerColour::Black,
            _ => return None
        };

        let castle_rights = input_parts.next()?;
        let white_ks_castle = castle_rights.contains('K');
        let white_qs_castle = castle_rights.contains('Q');
        let black_ks_castle = castle_rights.contains('k');
        let black_qs_castle = castle_rights.contains('q');

        let enpasant_col = match input_parts.next()?.chars().next()? {
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
        let meta = GSMetaData { white_ks_castle, black_ks_castle, white_qs_castle, black_qs_castle, enpasant_col };
        Some(GameState { turn, board, meta })
    }

    fn move_alg(&mut self, _mov: &AlgebraicMove) {
        todo!()
    }

    fn move_det(&mut self, mov: &FesMoveDet) {
        let (fx, fy) = unpack_index(mov.from);
        let (tx, ty) = unpack_index(mov.to);
        if mov.from == 0 || mov.to == 0 || mov.from == 4 { // mov to 4 without moving from 4 would be taking the king
            self.meta.white_qs_castle = false
        }
        if mov.from == 4 || mov.to == 4 || mov.from == 4 { // mov to 4 without moving from 4 would be taking the king
            self.meta.white_ks_castle = false
        }
        if mov.from == 56 || mov.to == 56 || mov.from == 60 { // mov to 60 without moving from 60 would be taking the king
            self.meta.black_qs_castle = false
        }
        if mov.from == 63 || mov.to == 63 || mov.from == 60 { // mov to 60 without moving from 60 would be taking the king
            self.meta.black_ks_castle = false
        }
        if self.board.pieces[fy][fx].unwrap().piece() == Piece::Pawn &&
            ((fy == 1 && ty == 3) || (fy == 6 && ty == 4)) {
            self.meta.enpasant_col = Some(fx as u8);
        }
        else {
            self.meta.enpasant_col = None;
        }
        //unwrap should be fine as move should be from a piece
        if self.board.pieces[fy][fx].unwrap().piece() == Piece::King {
            if fx == 4 && tx == 6 {
                debug_assert!(fy == 0 || fy == 7);
                self.board.pieces[fy][5] = Some(ColouredPiece::from_parts(self.turn, Piece::Rook));
                self.board.pieces[fy][7] = None;
            }
            else if fx == 4 && tx == 2 {
                debug_assert!(fy == 0 || fy == 7);
                self.board.pieces[fy][3] = Some(ColouredPiece::from_parts(self.turn, Piece::Rook));
                self.board.pieces[fy][0] = None;
            }
        }
        if mov.enpas {
            if ty == 2 {
                self.board.pieces[3][tx] = None;
            }
            if ty == 5 {
                self.board.pieces[4][tx] = None;
            }
        }
        self.board.pieces[ty][tx] = match mov.promo {
            Some(p) => Some(ColouredPiece::from_parts(self.turn, p)),
            None => self.board.pieces[fy][fx],
        };
        assert!(self.board.pieces[fy][fx].is_some());
        assert!(self.board.pieces[ty][tx].is_some());
        self.board.pieces[fy][fx] = None;
        match self.turn {
            White => self.turn = Black,
            Black => self.turn = White,
        }
    }

    fn unmove_det(&mut self, mov: &FesMoveDet) {
        let (fx, fy) = unpack_index(mov.from);
        let (tx, ty) = unpack_index(mov.to);
        match self.turn {
            White => self.turn = Black,
            Black => self.turn = White,
        }
        if self.board.pieces[ty][tx].is_none() {
            println!("{}", self);
            println!("{:?}", mov);
        }
        if self.board.pieces[ty][tx].unwrap().piece() == Piece::King {
            if fx == 4 && tx == 6 {
                debug_assert!(fy == 0 || fy == 7);
                self.board.pieces[fy][7] = Some(ColouredPiece::from_parts(self.turn, Piece::Rook));
                self.board.pieces[fy][5] = None;
            }
            else if fx == 4 && tx == 2 {
                debug_assert!(fy == 0 || fy == 7);
                self.board.pieces[fy][0] = Some(ColouredPiece::from_parts(self.turn, Piece::Rook));
                self.board.pieces[fy][3] = None;
            }
        }

        if mov.enpas {
            if ty == 2 {
                self.board.pieces[3][tx] = Some(WhitePawn);
            }
            if ty == 5 {
                self.board.pieces[4][tx] = Some(BlackPawn);
            }
        }
        self.board.pieces[fy][fx] = match mov.promo {
            Some(_) => Some(ColouredPiece::from_parts(self.turn, Piece::Pawn)),
            None => self.board.pieces[ty][tx],
        };
        self.board.pieces[ty][tx] = match mov.take {
            Some(p) => Some(ColouredPiece::from_parts(self.turn.invert(), p)),
            None => None,
        };
        self.meta = mov.meta.clone();
    }

    fn moves(&mut self) -> Vec<FesMoveDet> {
        let moves = self.get_preliminary_moves();
        let moves: Vec<_>= moves.into_iter().filter(|mov| self.validate_move(mov)).collect();
        moves
    }
}

impl GameState {
    /// return true if the move was legal and didnt take a piece
    /// (if false sliding peices cant take another step)
    fn optionaly_add(&self, col: PlayerColour, old_x: usize, old_y: usize, new_x: usize, new_y: usize, moves: &mut Vec<FesMoveDet>) -> bool {
        if legal_pos(new_x, new_y) && !ColouredPiece::opt_is_col(self.board.pieces[new_y][new_x], col, false) {
            let take = match self.board.pieces[new_y][new_x] {
                Some(p) => Some(p.piece()),
                None => None,
            };
            let from = pack(old_x, old_y);
            let to = pack(new_x, new_y);
            FesMoveDet::push_take(moves, from, to, take, &self.meta);
            return take.is_none();
        }
        return false;
    }

    /// rrook moves
    fn rook_moves(&self, col: PlayerColour, x: usize, y: usize,
            moves: &mut Vec<FesMoveDet>) {
        let mut dist = 1;
        while self.optionaly_add(col, x, y, x + dist, y, moves) {
            dist += 1
        }
        let mut dist = 1;
        while self.optionaly_add(col, x, y, x, y + dist, moves) {
            dist += 1
        }
        let mut dist = 1;
        while self.optionaly_add(col, x, y, x - dist, y, moves) {
            dist += 1
        }
        let mut dist = 1;
        while self.optionaly_add(col, x, y, x, y - dist, moves) {
            dist += 1
        }
    }

    /// bishop moves
    fn bishop_moves(&self, col: PlayerColour, x: usize, y: usize,
            moves: &mut Vec<FesMoveDet>) {
        let mut dist = 1;
        while self.optionaly_add(col, x, y, x + dist, y + dist, moves) {
            dist += 1
        }
        let mut dist = 1;
        while self.optionaly_add(col, x, y, x - dist, y + dist, moves) {
            dist += 1
        }
        let mut dist = 1;
        while self.optionaly_add(col, x, y, x + dist, y - dist, moves) {
            dist += 1
        }
        let mut dist = 1;
        while self.optionaly_add(col, x, y, x - dist, y - dist, moves) {
            dist += 1
        }
    }

    fn get_preliminary_moves(&self) -> Vec<FesMoveDet> {
        let mut white_moves = Vec::new();
        let mut black_moves = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                match self.board.pieces[y][x] {
                    Some(WhitePawn) => {
                        if x > 0 && (ColouredPiece::opt_is_black(self.board.pieces[y + 1][x - 1], false) ||
                                (y == 4 && self.meta.enpasant_col.is_some_and(|col| col as usize == x - 1))) {
                            let take = match self.board.pieces[y + 1][x - 1] {
                                Some(p) => Some(p.piece()),
                                None => None
                            };
                            let from = pack(x, y);
                            let to = pack(x - 1, y + 1);
                            if y == 6 {
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Queen, take, &self.meta);
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Rook, take, &self.meta);
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Knight, take, &self.meta);
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Bishop, take, &self.meta);
                            }
                            else {
                                if y == 4 && self.meta.enpasant_col.is_some_and(|col| col as usize == x - 1) {
                                    FesMoveDet::push_enpas(&mut white_moves, from, to, &self.meta)
                                }
                                else {
                                    FesMoveDet::push_take(&mut white_moves, from, to, take, &self.meta);
                                }
                            }
                        }
                        if x < 7 && (ColouredPiece::opt_is_black(self.board.pieces[y + 1][x + 1], false) ||
                                (y == 4 && self.meta.enpasant_col.is_some_and(|col| col as usize == x + 1))) {
                            let take = match self.board.pieces[y + 1][x + 1] {
                                Some(p) => Some(p.piece()),
                                None => None
                            };
                            let from = pack(x, y);
                            let to = pack(x + 1, y + 1);
                            if y == 6 {
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Queen, take, &self.meta);
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Rook, take, &self.meta);
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Knight, take, &self.meta);
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Bishop, take, &self.meta);
                            }
                            else {
                                if y == 4 && self.meta.enpasant_col.is_some_and(|col| col as usize == x + 1) {
                                    FesMoveDet::push_enpas(&mut white_moves, from, to, &self.meta)
                                }
                                else {
                                    FesMoveDet::push_take(&mut white_moves, from, to, take, &self.meta);
                                }
                            }
                        }
                        if self.board.pieces[y + 1][x].is_none() {
                            let from = pack(x, y);
                            let to = pack(x, y + 1);
                            if y == 6 {
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Queen, None, &self.meta);
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Rook, None, &self.meta);
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Knight, None, &self.meta);
                                FesMoveDet::push_promo(&mut white_moves, from, to, Piece::Bishop, None, &self.meta);
                            }
                            else {
                                FesMoveDet::push_basic(&mut white_moves, from, to, &self.meta);
                            }
                            if y == 1 && self.board.pieces[y + 2][x].is_none() {
                                let from = pack(x, y);
                                let to = pack(x, y + 2);
                                FesMoveDet::push_basic(&mut white_moves, from, to, &self.meta);
                            }
                        }
                    },
                    Some(WhiteKnight) => {
                        for di in 1..=2 {
                            let dj = 3 - di;
                            self.optionaly_add(White, x, y, x + di, y + dj, &mut white_moves);
                            self.optionaly_add(White, x, y, x - di, y + dj, &mut white_moves);
                            self.optionaly_add(White, x, y, x + di, y - dj, &mut white_moves);
                            self.optionaly_add(White, x, y, x - di, y - dj, &mut white_moves);
                        }
                    },
                    Some(WhiteBishop) => {
                        self.bishop_moves(White, x, y, &mut white_moves);
                    },
                    Some(WhiteRook) => {
                        self.rook_moves(White, x, y, &mut white_moves);
                    },
                    Some(WhiteQueen) => {
                        self.bishop_moves(White, x, y, &mut white_moves);
                        self.rook_moves(White, x, y, &mut white_moves);
                    },
                    Some(WhiteKing) => {
                        self.optionaly_add(White, x, y, x + 1, y + 1, &mut white_moves);
                        self.optionaly_add(White, x, y, x + 1,     y, &mut white_moves);
                        self.optionaly_add(White, x, y, x + 1, y - 1, &mut white_moves);
                        self.optionaly_add(White, x, y,     x, y + 1, &mut white_moves);
                        self.optionaly_add(White, x, y,     x, y - 1, &mut white_moves);
                        self.optionaly_add(White, x, y, x - 1, y + 1, &mut white_moves);
                        self.optionaly_add(White, x, y, x - 1,     y, &mut white_moves);
                        self.optionaly_add(White, x, y, x - 1, y - 1, &mut white_moves);
                    },
                    Some(BlackPawn) => {
                        if x > 0 && (ColouredPiece::opt_is_white(self.board.pieces[y - 1][x - 1], false) ||
                                (y == 3 && self.meta.enpasant_col.is_some_and(|col| col as usize == x - 1))) {
                            let take = match self.board.pieces[y - 1][x - 1] {
                                Some(p) => Some(p.piece()),
                                None => None
                            };
                            let from = pack(x, y);
                            let to = pack(x - 1, y - 1);
                            if y == 1 {
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Queen, take, &self.meta);
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Rook, take, &self.meta);
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Knight, take, &self.meta);
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Bishop, take, &self.meta);
                            }
                            else {
                                if y == 3 && self.meta.enpasant_col.is_some_and(|col| col as usize == x - 1) {
                                    FesMoveDet::push_enpas(&mut black_moves, from, to, &self.meta)
                                }
                                else {
                                    FesMoveDet::push_take(&mut black_moves, from, to, take, &self.meta);
                                }
                            }
                        }
                        if x < 7 && (ColouredPiece::opt_is_white(self.board.pieces[y - 1][x + 1], false) ||
                                (y == 3 && self.meta.enpasant_col.is_some_and(|col| col as usize == x + 1))) {
                            let take = match self.board.pieces[y - 1][x + 1] {
                                Some(p) => Some(p.piece()),
                                None => None
                            };
                            let from = pack(x, y);
                            let to = pack(x + 1, y - 1);
                            if y == 1 {
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Queen, take, &self.meta);
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Rook, take, &self.meta);
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Knight, take, &self.meta);
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Bishop, take, &self.meta);
                            }
                            else {
                                if y == 3 && self.meta.enpasant_col.is_some_and(|col| col as usize == x + 1) {
                                    FesMoveDet::push_enpas(&mut black_moves, from, to, &self.meta)
                                }
                                else {
                                    FesMoveDet::push_take(&mut black_moves, from, to, take, &self.meta);
                                }
                            }
                        }
                        if self.board.pieces[y - 1][x].is_none() {
                            let from = pack(x, y);
                            let to = pack(x, y - 1);
                            if y == 1 {
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Queen, None, &self.meta);
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Rook, None, &self.meta);
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Knight, None, &self.meta);
                                FesMoveDet::push_promo(&mut black_moves, from, to, Piece::Bishop, None, &self.meta);
                            }
                            else {
                                FesMoveDet::push_basic(&mut black_moves, from, to, &self.meta);
                            }
                            if y == 6 && self.board.pieces[y - 2][x].is_none() {
                                let from = pack(x, y);
                                let to = pack(x, y - 2);
                                FesMoveDet::push_basic(&mut black_moves, from, to, &self.meta);
                            }
                        }
                    },
                    Some(BlackKnight) => {
                        for di in 1..=2 {
                            let dj = 3 - di;
                            self.optionaly_add(Black, x, y, x + di, y + dj, &mut black_moves);
                            self.optionaly_add(Black, x, y, x - di, y + dj, &mut black_moves);
                            self.optionaly_add(Black, x, y, x + di, y - dj, &mut black_moves);
                            self.optionaly_add(Black, x, y, x - di, y - dj, &mut black_moves);
                        }
                    },
                    Some(BlackBishop) => {
                        self.bishop_moves(Black, x, y, &mut black_moves);
                    }
                    Some(BlackRook) => {
                        self.rook_moves(Black, x, y, &mut black_moves);
                    }
                    Some(BlackQueen) => {
                        self.bishop_moves(Black, x, y, &mut black_moves);
                        self.rook_moves(Black, x, y, &mut black_moves);
                    }
                    Some(BlackKing) => {
                        self.optionaly_add(Black, x, y, x + 1, y + 1, &mut black_moves);
                        self.optionaly_add(Black, x, y, x + 1,     y, &mut black_moves);
                        self.optionaly_add(Black, x, y, x + 1, y - 1, &mut black_moves);
                        self.optionaly_add(Black, x, y,     x, y + 1, &mut black_moves);
                        self.optionaly_add(Black, x, y,     x, y - 1, &mut black_moves);
                        self.optionaly_add(Black, x, y, x - 1, y + 1, &mut black_moves);
                        self.optionaly_add(Black, x, y, x - 1,     y, &mut black_moves);
                        self.optionaly_add(Black, x, y, x - 1, y - 1, &mut black_moves);
                    },
                    None => {/* Section intentionally left blank */},
                }
            }
        }
        if self.meta.white_ks_castle && !black_moves.iter().any(|mov| mov.to >=4 && mov.to <= 6) &&
            !self.board.pieces[1][4..=7].iter().any(|p| *p == Some(BlackPawn)) &&
            !self.board.pieces[0][5..=6].iter().any(|p| p.is_some())  {
            FesMoveDet::push_basic(&mut white_moves, 4, 6, &self.meta);
        }
        if self.meta.white_qs_castle && !black_moves.iter().any(|mov| mov.to >= 2 && mov.to <= 4) &&
            !self.board.pieces[1][1..=4].iter().any(|p| *p == Some(BlackPawn)) &&
            !self.board.pieces[0][1..=3].iter().any(|p| p.is_some())  {
            FesMoveDet::push_basic(&mut white_moves, 4, 2, &self.meta);
        }

        if self.meta.black_ks_castle && !white_moves.iter().any(|mov| mov.to >=60 && mov.to <= 62) &&
            !self.board.pieces[6][4..=7].iter().any(|p| *p == Some(WhitePawn)) &&
            !self.board.pieces[7][5..=6].iter().any(|p| p.is_some())  {
            FesMoveDet::push_basic(&mut black_moves, 60, 62, &self.meta);
        }
        if self.meta.black_qs_castle && !white_moves.iter().any(|mov| mov.to >= 58 && mov.to <= 60) &&
            !self.board.pieces[6][1..=4].iter().any(|p| *p == Some(WhitePawn)) &&
            !self.board.pieces[7][1..=3].iter().any(|p| p.is_some())  {
            FesMoveDet::push_basic(&mut black_moves, 60, 58, &self.meta);
        }

        match self.turn {
            White => white_moves,
            Black => black_moves,
        }
    }

    fn validate_move(&mut self, mov: &FesMoveDet) -> bool {
        self.move_det(mov);
        let prelim_moves = self.get_preliminary_moves();
        self.unmove_det(mov);
        let a = !prelim_moves.iter().any(|mov| if let Some(Piece::King) = mov.take {true} else {false});
        a
    }
}
