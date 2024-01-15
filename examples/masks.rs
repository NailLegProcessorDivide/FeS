use fes::{game::ChessGame, bit_board::{BitBoardGame, print_bitmask}};

fn main() {
    let gs = BitBoardGame::from_fen("8/7q/6P1/3p1p2/4K1Pq/8/8/8 w - - 0 1").unwrap();

    println!();
    println!("{}", gs.board);

    print_bitmask(gs.board.check_mask::<true>());

}