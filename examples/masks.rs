use fes::{game::ChessGame, bit_board::{BitBoardGame, print_bitmask}};

fn main() {
    let gs = BitBoardGame::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ").unwrap();

    print_bitmask(gs.board.attack_mask::<true>());
    println!();
    println!("{}", gs.board);
}