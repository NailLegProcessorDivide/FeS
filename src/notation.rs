use regex::Regex;

use crate::piece::Piece;

pub enum AlgebraicPosition {
    Square(u8, u8),
    Piece(Piece),
    RankPiece(u8, Piece),
    FilePiece(u8, Piece),
    SquarePiece(u8, u8, Piece),
}

pub enum AlgebraicMove {
    Move(AlgebraicPosition, AlgebraicPosition),
    Promotion(AlgebraicPosition, AlgebraicPosition, Piece),
    KSCastle,
    QSCastle,
}

/// Function to map squares in standard string representation to row, col index
/// in: square string
/// out: (rank, file) indicies
///  e.g. "a4" -> (3, 0)
///  e.g. "h7" -> (6, 7)
pub fn parse_square(inp: &str) -> Option<(u8, u8)> {
    if inp.len() != 2 {
        return None;
    }
    let mut chrs = inp.chars();
    let file = chrs.next().unwrap();
    let rank = chrs.next().unwrap();
    if file >= 'a' && file <= 'h' && rank >= '1' && rank <= '8' {
        return Some((rank as u8 - '1' as u8, file as u8 - 'a' as u8));
    }
    return None;
}

pub fn parse_rank(inp: &str) -> Option<u8> {
    if inp.len() != 1 {
        return None;
    }
    let mut chrs = inp.chars();
    let rank = chrs.next().unwrap();
    if rank >= '1' && rank <= '8' {
        return Some(rank as u8 - '1' as u8);
    }
    return None;
}

pub fn parse_file(inp: &str) -> Option<u8> {
    if inp.len() != 1 {
        return None;
    }
    let mut chrs = inp.chars();
    let file = chrs.next().unwrap();
    if file >= 'a' && file <= 'h' {
        return Some(file as u8 - 'a' as u8);
    }
    return None;
}

pub const fn parse_piece_letter(inp: char) -> Option<Piece> {
    match inp {
        'P' => Some(Piece::Pawn),
        'N' => Some(Piece::Knight),
        'B' => Some(Piece::Bishop),
        'R' => Some(Piece::Rook),
        'Q' => Some(Piece::Queen),
        'K' => Some(Piece::King),
        _ => None
    }
}

use AlgebraicMove::*;
use AlgebraicPosition::*;

pub fn str_to_algebraic(inp: &str) -> Option<AlgebraicMove> {
    let inp = inp.trim();
    let re = Regex::new(r"([NBRQK])?([a-h])?([1-8])?x?([a-h][1-8])(=[BNRQ])?#?\+?").ok()?;
    Some(
        if inp == "O-O" {
            AlgebraicMove::KSCastle
        }
        else if inp == "O-O-O" {
            AlgebraicMove::QSCastle
        }
        else if let Some(caps) = re.captures(inp) {
            let moving_piece_type = match caps.get(1) {
                None => Piece::Pawn,
                Some(piece) => parse_piece_letter(piece.as_str().chars().next()?)?
            };
            let moving_piece = match (caps.get(2), caps.get(3)) {
                (Some(f), Some(r)) => SquarePiece(
                                        parse_rank(r.as_str())?,
                                        parse_file(f.as_str())?,
                                        moving_piece_type),
                (Some(f), None) => FilePiece(
                                        parse_file(f.as_str())?,
                                        moving_piece_type),
                (None, Some(r)) => RankPiece(
                                        parse_rank(r.as_str())?,
                                        moving_piece_type),
                (None, None) => Piece(moving_piece_type),
            };
            match (caps.get(4)?, caps.get(5)) {
                (sqr, None) => {
                    let (r, f) = parse_square(sqr.as_str())?;
                    Move(moving_piece, Square(r, f))
                }
                (sqr, Some(promo)) => {
                    assert!(caps.get(1).is_none());
                    let (r, f) = parse_square(sqr.as_str())?;
                    Promotion(Piece(Piece::Pawn), Square(r, f), parse_piece_letter(promo.as_str().chars().nth(2)?)?)
                }
            }
        }
        else {
            panic!("unkown move \"{inp}\"")
        }
    )
}
