use crate::*;

#[test]
fn should_create_a_new_board() {
    fn is_empty(board: &Board) -> bool {
        let mut empty = true;
        for row in board.cells {
            for cell in row {
                if cell != Cell::Empty {
                    empty = false;
                }
            }
        }
        empty
    }

    let board = Board::new();
    assert_eq!(board.cells.len(), BOARD_HEIGHT);
    assert_eq!(board.cells[0].len(), BOARD_WIDTH);
    assert!(is_empty(&board))
}

#[test]
fn board_cell_can_be_changed() {
    let mut board = Board::new();
    let coords = Coordinates { row: 5, col: 5 };

    assert_eq!(
        board.get_cell(&coords),
        Cell::Empty,
        "Cell should be empty before change."
    );
    board.change_cell(coords, Cell::Block);
    assert_eq!(
        board.get_cell(&coords),
        Cell::Block,
        "Cell should had changed."
    );
}

#[test]
fn should_create_new_game() {
    let game_state = Game::new_game();
    assert_eq!(
        game_state.phase,
        GamePhase::Bomb,
        "The game should start in bomb phase"
    );
    assert_eq!(game_state.winner, None, "No player should have won yet");
    assert_eq!(game_state.next_player, 0);
    assert_eq!(game_state.bombs.len(), NUM_OF_PLAYERS);
    assert!(
        game_state
            .bombs
            .iter()
            .all(|bombs| { *bombs == NUM_OF_BOMBS_PER_PLAYER }),
        "Each player should have {NUM_OF_BOMBS_PER_PLAYER} bombs"
    );
}

#[test]
fn a_board_in_a_new_game_should_contain_random_blocks() {
    let state = Game::new_game();
    let mut blocks = 0;

    // Count all cells of type 'block' in board.
    for i in 0..BOARD_HEIGHT {
        for j in 0..BOARD_WIDTH {
            if state.board.get_cell(&Coordinates { row: i, col: j }) == Cell::Block {
                blocks += 1;
            }
        }
    }

    assert_eq!(blocks, NUM_OF_BLOCKS);
}

#[test]
fn a_player_cannot_drop_bomb_in_play_phase() {
    let mut game_state = Game::new_game();
    game_state.phase = GamePhase::Play;
    let result = Game::drop_bomb(game_state, Coordinates { row: 0, col: 0 }, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GameError::DroppedBombDuringPlayPhase);
}

#[test]
fn a_player_cannot_drop_bomb_if_already_dropped_all() {
    let mut game_state = Game::new_game();
    game_state.board = Board::new();
    game_state.bombs = [0, 0];
    let result = Game::drop_bomb(game_state, Coordinates { row: 0, col: 0 }, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GameError::NoMoreBombsAvailable);
}

#[test]
fn a_player_drops_a_bomb() {
    let mut game_state = Game::new_game();
    game_state.board = Board::new();

    let player = 0;
    let bomb_position = Coordinates { row: 0, col: 0 };
    let player_bombs = game_state.bombs[player as usize];
    assert_eq!(player_bombs, NUM_OF_BOMBS_PER_PLAYER);

    let drop_bomb_result = Game::drop_bomb(game_state, bomb_position.clone(), player);
    assert!(drop_bomb_result.is_ok());
    let game_state = drop_bomb_result.unwrap();

    assert_eq!(
        game_state.bombs[player as usize],
        player_bombs - 1,
        "The player should have one bomb less available for dropping"
    );
    assert_eq!(
        game_state.board.get_cell(&bomb_position),
        Cell::Bomb([Some(player), None])
    )
}

#[test]
fn a_cell_can_hold_one_or_more_bombs_from_different_players() {
    let mut game_state = Game::new_game();
    game_state.board = Board::new();
    let bomb_position = Coordinates { row: 0, col: 0 };
    let player1 = 0;
    let player2 = 1;

    let drop_bomb_result = Game::drop_bomb(game_state, bomb_position.clone(), player1);
    assert!(drop_bomb_result.is_ok());
    let game_state = drop_bomb_result.unwrap();

    assert_eq!(
        game_state.board.get_cell(&bomb_position),
        Cell::Bomb([Some(player1), None])
    );

    let drop_bomb_result = Game::drop_bomb(game_state, bomb_position.clone(), player2);
    assert!(drop_bomb_result.is_ok());
    let game_state = drop_bomb_result.unwrap();
    assert_eq!(
        game_state.board.get_cell(&bomb_position),
        Cell::Bomb([Some(player1), Some(player2)])
    );
}

#[test]
fn a_bomb_cannot_be_placed_in_a_block_cell() {
    let mut game_state = Game::new_game();
    game_state.board.cells[0][0] = Cell::Block;
    let result = Game::drop_bomb(game_state, Coordinates { row: 0, col: 0 }, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GameError::InvalidBombPosition);
}

#[test]
fn a_player_cannot_place_more_than_one_bomb_in_a_cell() {
    let mut game_state = Game::new_game();
    game_state.board = Board::new();
    let position = Coordinates { row: 0, col: 0 };
    let player = 0;

    // Drop the first bomb
    let dropping_bomb_result = Game::drop_bomb(game_state, position, player);
    assert!(
        dropping_bomb_result.is_ok(),
        "Dropping the first bomb should be OK"
    );
    let game_state = dropping_bomb_result.unwrap();
    assert_eq!(
        game_state.board.get_cell(&position),
        Cell::Bomb([Some(0), None])
    );

    // Drop the second bomb
    let dropping_bomb_result = Game::drop_bomb(game_state, position, player);
    assert!(
        dropping_bomb_result.is_err(),
        "Dropping the second bomb should be Err"
    );
    assert_eq!(
        dropping_bomb_result.unwrap_err(),
        GameError::InvalidBombPosition
    );
}

#[test]
fn a_game_can_change_game_phase() {
    let game_state = Game::new_game();
    assert_eq!(game_state.phase, GamePhase::Bomb);
    let game_state = Game::change_game_phase(game_state, GamePhase::Play);
    assert_eq!(game_state.phase, GamePhase::Play);
    let game_state = Game::change_game_phase(game_state, GamePhase::Bomb);
    assert_eq!(game_state.phase, GamePhase::Bomb);
}

#[test]
fn a_player_cannot_drop_a_stone_out_of_turn() {
    let state = Game::new_game();
    let drop_stone_result = Game::drop_stone(state, 1, Side::North, 0);
    assert!(drop_stone_result.is_err());
    assert_eq!(drop_stone_result.unwrap_err(), GameError::NotPlayerTurn);
}

#[test]
fn a_stone_dropped_from_north_side_should_move_until_it_reaches_an_obstacle() {
    let o = Cell::Empty;
    let b = Cell::Block;
    let cells = [
        [o, o, o, b, o, o, o, o, o, o],
        [o, o, b, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, b, o, o, o, o, o, o, o, o],
    ];

    let mut state = Game::new_game();
    state.board.cells = cells;
    let player = 0;

    let state = Game::drop_stone(state, player, Side::North, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 0 }),
        Cell::Stone(player)
    );
    let state = Game::drop_stone(state, player, Side::North, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 8, col: 1 }),
        Cell::Stone(player)
    );
    let state = Game::drop_stone(state, player, Side::North, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 2 }),
        Cell::Stone(player)
    );
    assert_eq!(
        Game::drop_stone(state, player, Side::North, 3).unwrap_err(),
        GameError::InvalidDroppingPosition
    );
}

#[test]
fn a_stone_dropped_from_south_side_should_move_until_it_reaches_an_obstacle() {
    let o = Cell::Empty;
    let b = Cell::Block;

    let cells = [
        [o, b, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, b, o, o, o, o, o, o, o],
        [o, o, o, b, o, o, o, o, o, o],
    ];

    let mut state = Game::new_game();
    state.board.cells = cells;
    let player = 0;

    let state = Game::drop_stone(state, player, Side::South, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 0 }),
        Cell::Stone(player)
    );
    let state = Game::drop_stone(state, player, Side::South, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 1, col: 1 }),
        Cell::Stone(player)
    );
    let state = Game::drop_stone(state, player, Side::South, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 2 }),
        Cell::Stone(player)
    );
    assert_eq!(
        Game::drop_stone(state, player, Side::South, 3).unwrap_err(),
        GameError::InvalidDroppingPosition
    );
}

#[test]
fn a_stone_dropped_from_east_side_should_move_until_it_reaches_an_obstacle() {
    let o = Cell::Empty;
    let b = Cell::Block;

    let cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, b, o],
        [o, o, o, o, o, o, o, o, o, b],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    let mut state = Game::new_game();
    state.board.cells = cells;
    let player = 0;

    let state = Game::drop_stone(state, player, Side::East, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 0 }),
        Cell::Stone(player)
    );
    let state = Game::drop_stone(state, player, Side::East, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 1, col: 1 }),
        Cell::Stone(player)
    );
    let state = Game::drop_stone(state, player, Side::East, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 2, col: 9 }),
        Cell::Stone(player)
    );
    assert_eq!(
        Game::drop_stone(state, player, Side::East, 3).unwrap_err(),
        GameError::InvalidDroppingPosition
    );
}
#[test]
fn a_stone_dropped_from_west_side_should_move_until_it_reaches_an_obstacle() {
    let o = Cell::Empty;
    let b = Cell::Block;

    let cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, b],
        [o, b, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    let mut state = Game::new_game();
    state.board.cells = cells;
    let player = 0;

    let state = Game::drop_stone(state, player, Side::West, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 9 }),
        Cell::Stone(player)
    );
    let state = Game::drop_stone(state, player, Side::West, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 1, col: 8 }),
        Cell::Stone(player)
    );
    let state = Game::drop_stone(state, player, Side::West, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 2, col: 0 }),
        Cell::Stone(player)
    );
    assert_eq!(
        Game::drop_stone(state, player, Side::West, 3).unwrap_err(),
        GameError::InvalidDroppingPosition
    );
}

#[test]
fn a_stone_should_explode_a_bomb_when_passing_through() {
    let o = Cell::Empty;
    let b = Cell::Bomb([Some(0), Some(1)]);
    let x = Cell::Stone(0);
    let l = Cell::Block;

    let mut state = Game::new_game();
    state.board.cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, b, o, o, o, o, o, o],
        [o, o, o, o, x, b, l, o, o, o],
        [o, o, o, o, b, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    let dropping_stone_result = Game::drop_stone(state, 0, Side::North, 5);
    assert!(dropping_stone_result.is_ok());
    let board = dropping_stone_result.unwrap().board;

    // Stone in position 3,4 should be destroyed.
    assert_eq!(board.get_cell(&Coordinates { row: 3, col: 4 }), Cell::Empty);

    // Bomb in position 4,4 should be destroyed.
    assert_eq!(board.get_cell(&Coordinates { row: 4, col: 4 }), Cell::Empty);

    // Block in position 3,6 should not be destroyed.
    assert_eq!(board.get_cell(&Coordinates { row: 3, col: 6 }), Cell::Block);

    // Bomb in position 2,3 should not be destroyed.
    assert_eq!(
        board.get_cell(&Coordinates { row: 2, col: 3 }),
        Cell::Bomb([Some(0), Some(1)])
    );
}

#[test]
fn a_player_wins_when_has_a_four_stone_vertical_row() {
    let player = 0;
    let o = Cell::Empty;
    let s = Cell::Stone(player);

    let mut state = Game::new_game();
    state.board.cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, s, o, o, o, o],
        [o, o, o, o, o, s, o, o, o, o],
        [o, o, o, o, o, s, o, o, o, o],
        [o, o, o, o, o, s, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    state = Game::check_winner_player(state);
    assert!(state.winner.is_some());
    assert_eq!(state.winner.unwrap(), player);
}

#[test]
fn a_player_wins_when_has_a_four_stone_horizontal_row() {
    let player = 0;
    let o = Cell::Empty;
    let s = Cell::Stone(player);

    let mut state = Game::new_game();
    state.board.cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, s, s, s, s, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    state = Game::check_winner_player(state);
    assert!(state.winner.is_some());
    assert_eq!(state.winner.unwrap(), player);
}

#[test]
fn a_player_wins_when_has_a_four_stone_ascending_diagonal_row() {
    let player = 0;
    let o = Cell::Empty;
    let s = Cell::Stone(player);

    let mut state = Game::new_game();
    state.board.cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, s, o, o, o, o, o],
        [o, o, o, s, o, o, o, o, o, o],
        [o, o, s, o, o, o, o, o, o, o],
        [o, s, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    state = Game::check_winner_player(state);
    assert!(state.winner.is_some());
    assert_eq!(state.winner.unwrap(), player);
}

#[test]
fn a_player_wins_when_has_a_four_stone_descending_diagonal_row() {
    let player = 0;
    let o = Cell::Empty;
    let s = Cell::Stone(player);

    let mut state = Game::new_game();
    state.board.cells = [
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, s, o, o, o, o, o],
        [o, o, o, o, o, s, o, o, o, o],
        [o, o, o, o, o, o, s, o, o, o],
        [o, o, o, o, o, o, o, s, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    state = Game::check_winner_player(state);
    assert!(state.winner.is_some());
    assert_eq!(state.winner.unwrap(), player);
}

#[test]
fn no_player_wins_for_less_than_four_in_a_row_stones() {
    let player1 = 0;
    let player2 = 1;
    let o = Cell::Empty;
    let b = Cell::Block;
    let r = Cell::Stone(player1);
    let m = Cell::Stone(player2);

    let mut state = Game::new_game();
    state.board.cells = [
        [o, r, o, o, o, o, o, o, m, o],
        [m, o, o, o, o, o, o, o, o, o],
        [m, o, r, r, m, m, m, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [m, o, o, o, r, o, o, o, o, o],
        [m, m, b, m, o, r, o, o, o, o],
        [o, o, o, o, b, o, m, o, o, o],
        [o, o, r, o, o, o, o, r, o, o],
        [o, r, r, o, o, o, o, o, o, o],
        [r, r, r, o, o, o, o, o, o, o],
    ];

    state = Game::check_winner_player(state);
    assert!(state.winner.is_none(), "No player should have won");
}
