use std::fmt::Display;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayerColour {
    White = 0,
    Black = 1,
}

impl PlayerColour {
    pub fn invert(self) -> Self {
        match self {
            PlayerColour::White => PlayerColour::Black,
            PlayerColour::Black => PlayerColour::White,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    pub fn is_col(self, col: PlayerColour) -> bool {
        match col {
            PlayerColour::White => todo!(),
            PlayerColour::Black => todo!(),
        }
    }
    pub fn opt_is_black(v: Option<Self>, none_val: bool) -> bool {
        match v {
            Some(v) => v as u8 & 0x8 != 0,
            None => none_val,
        }
    }
    pub fn opt_is_white(v: Option<Self>, none_val: bool) -> bool {
        match v {
            Some(v) => v as u8 & 0x8 == 0,
            None => none_val,
        }
    }
    pub fn opt_is_col(v: Option<Self>, col: PlayerColour, none_val: bool) -> bool {
        match col {
            PlayerColour::White => Self::opt_is_white(v, none_val),
            PlayerColour::Black => Self::opt_is_black(v, none_val),
        }
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
    pub fn from_parts(col: PlayerColour, p: Piece) -> Self {
        match (col, p) {
            (PlayerColour::White, Piece::Pawn) => ColouredPiece::WhitePawn,
            (PlayerColour::White, Piece::Knight) => ColouredPiece::WhiteKnight,
            (PlayerColour::White, Piece::Bishop) => ColouredPiece::WhiteBishop,
            (PlayerColour::White, Piece::Rook) => ColouredPiece::WhiteRook,
            (PlayerColour::White, Piece::Queen) => ColouredPiece::WhiteQueen,
            (PlayerColour::White, Piece::King) => ColouredPiece::WhiteKing,
            (PlayerColour::Black, Piece::Pawn) => ColouredPiece::BlackPawn,
            (PlayerColour::Black, Piece::Knight) => ColouredPiece::BlackKnight,
            (PlayerColour::Black, Piece::Bishop) => ColouredPiece::BlackBishop,
            (PlayerColour::Black, Piece::Rook) => ColouredPiece::BlackRook,
            (PlayerColour::Black, Piece::Queen) => ColouredPiece::BlackQueen,
            (PlayerColour::Black, Piece::King) => ColouredPiece::BlackKing,
        }
    }
}

impl Display for ColouredPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
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
        })
    }
}
