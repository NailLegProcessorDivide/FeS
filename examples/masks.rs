use fes::{
    bit_board::{print_bitmask, BitBoardGame},
    game::ChessGame,
};

fn main() {
    let mut gs = BitBoardGame::from_fen("rnQqkbnr/p1p1pppp/8/1p1p4/8/4P3/PPPP1PPP/RNB1KBNR b KQkq - 0 3").unwrap();

    println!();

    let enemies = gs.board.col_piece_mask::<{!false}>();
    let other_attacks = gs.board.attack_mask::<{!false}>();
    let base_mask = gs.board.enemy_or_empty::<false>() & !other_attacks;
    let king = gs.board.col_king_mask::<false>();

    let from_idx = king.trailing_zeros() as u8;
    let mut to_mask = gs.board.king_attack_mask::<false>() & base_mask;

    print_bitmask(gs.board.hor_pin_mask::<false>());

    println!("\n\n");


    gs.moves();
}
