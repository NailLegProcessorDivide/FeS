use std::fmt::Display;



#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PlayerColor {
    White = 0,
    Black = 1,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Clone, Copy)]
pub enum ColouredPiece {
    WhitePawn = 0,
    WhiteKnight = 1,
    WhiteBishop = 2,
    WhiteRook = 3,
    WhiteQueen = 4,
    WhiteKing = 5,
    BlackPawn = 8,
    BlackKnight = 9,
    BlackBishop = 10,
    BlackRook = 11,
    BlackQueen = 12,
    BlackKing = 13,
}

impl ColouredPiece {
    pub fn is_black(self) -> bool {
        self as u8 & 0x8 != 0
    }
    pub fn is_white(self) -> bool {
        self as u8 & 0x8 == 0
    }
    pub fn opt_is_black(v: Option<Self>) -> bool {
        v.is_some() && v.unwrap() as u8 & 0x8 != 0
    }
    pub fn opt_is_white(v: Option<Self>) -> bool {
        v.is_some() && v.unwrap() as u8 & 0x8 == 0
    }
    pub fn piece(self) -> Piece {
        match self {
            ColouredPiece::BlackPawn | ColouredPiece::WhitePawn => Piece::Pawn,
            ColouredPiece::BlackKnight | ColouredPiece::WhiteKnight => Piece::Knight,
            ColouredPiece::BlackBishop | ColouredPiece::WhiteBishop => Piece::Bishop,
            ColouredPiece::BlackRook | ColouredPiece::WhiteRook => Piece::Rook,
            ColouredPiece::BlackQueen | ColouredPiece::WhiteQueen => Piece::Queen,
            ColouredPiece::BlackKing | ColouredPiece::WhiteKing => Piece::King,
        }
    }
    pub fn opt_piece(v: Option<Self>) -> Option<Piece> {
        match v {
            Some(p) => Some(p.piece()),
            None => None,
        }
    }
}

impl Display for ColouredPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                ColouredPiece::WhitePawn => "P",
                ColouredPiece::WhiteKnight => "N",
                ColouredPiece::WhiteBishop => "B",
                ColouredPiece::WhiteRook => "R",
                ColouredPiece::WhiteQueen => "Q",
                ColouredPiece::WhiteKing => "K",
                ColouredPiece::BlackPawn => "p",
                ColouredPiece::BlackKnight => "n",
                ColouredPiece::BlackBishop => "b",
                ColouredPiece::BlackRook => "r",
                ColouredPiece::BlackQueen => "q",
                ColouredPiece::BlackKing => "k",
            }
        )
    }
}

