// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::*;

const ALICE: u32 = 0;
const BOB: u32 = 1;

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
    board.update_cell(coords, Cell::Block);
    assert_eq!(
        board.get_cell(&coords),
        Cell::Block,
        "Cell should had changed."
    );
}

#[test]
fn should_create_new_game() {
    let game_state = Game::new_game(ALICE, BOB);
    assert_eq!(
        game_state.phase,
        GamePhase::Bomb,
        "The game should start in bomb phase"
    );
    assert_eq!(game_state.winner, None, "No player should have won yet");
    assert_eq!(game_state.next_player, ALICE);
    assert_eq!(game_state.bombs.len(), NUM_OF_PLAYERS);
    assert!(
        game_state
            .bombs
            .iter()
            .all(|bombs| { *bombs.1 == NUM_OF_BOMBS_PER_PLAYER }),
        "Each player should have {NUM_OF_BOMBS_PER_PLAYER} bombs"
    );
}

#[test]
fn a_board_in_a_new_game_should_contain_random_blocks() {
    let state = Game::new_game(ALICE, BOB);
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
    let mut game_state = Game::new_game(ALICE, BOB);
    game_state.phase = GamePhase::Play;
    let result = Game::drop_bomb(&mut game_state, Coordinates { row: 0, col: 0 }, ALICE);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GameError::DroppedBombDuringPlayPhase);
}

#[test]
fn a_player_cannot_drop_bomb_if_already_dropped_all() {
    let mut game_state = Game::new_game(ALICE, BOB);
    game_state.board = Board::new();
    game_state.bombs = HashMap::from([(0, 0)]);
    let result = Game::drop_bomb(&mut game_state, Coordinates { row: 0, col: 0 }, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GameError::NoMoreBombsAvailable);
}

#[test]
fn a_player_drops_a_bomb() {
    let mut game_state = Game::new_game(ALICE, BOB);
    game_state.board = Board::new();

    let bomb_position = Coordinates { row: 0, col: 0 };
    let player_bombs = game_state.bombs[&ALICE];
    assert_eq!(player_bombs, NUM_OF_BOMBS_PER_PLAYER);

    let drop_bomb_result = Game::drop_bomb(&mut game_state, bomb_position.clone(), ALICE);
    assert!(drop_bomb_result.is_ok());

    assert_eq!(
        game_state.bombs[&ALICE],
        player_bombs - 1,
        "The player should have one bomb less available for dropping"
    );
    assert_eq!(
        game_state.board.get_cell(&bomb_position),
        Cell::Bomb([Some(ALICE), None])
    )
}

#[test]
fn a_cell_can_hold_one_or_more_bombs_from_different_players() {
    let mut game_state = Game::new_game(ALICE, BOB);
    game_state.board = Board::new();
    let bomb_position = Coordinates { row: 0, col: 0 };

    let drop_bomb_result = Game::drop_bomb(&mut game_state, bomb_position.clone(), ALICE);
    assert!(drop_bomb_result.is_ok());

    assert_eq!(
        game_state.board.get_cell(&bomb_position),
        Cell::Bomb([Some(ALICE), None])
    );

    let drop_bomb_result = Game::drop_bomb(&mut game_state, bomb_position.clone(), BOB);
    assert!(drop_bomb_result.is_ok());

    assert_eq!(
        game_state.board.get_cell(&bomb_position),
        Cell::Bomb([Some(ALICE), Some(BOB)])
    );
}

#[test]
fn a_bomb_cannot_be_placed_in_a_block_cell() {
    let mut game_state = Game::new_game(ALICE, BOB);
    game_state.board.cells[0][0] = Cell::Block;
    let result = Game::drop_bomb(&mut game_state, Coordinates { row: 0, col: 0 }, ALICE);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), GameError::InvalidBombPosition);
}

#[test]
fn a_player_cannot_place_more_than_one_bomb_in_a_cell() {
    let mut game_state = Game::new_game(ALICE, BOB);
    game_state.board = Board::new();
    let position = Coordinates { row: 0, col: 0 };

    // Drop the first bomb
    let dropping_bomb_result = Game::drop_bomb(&mut game_state, position, ALICE);
    assert!(
        dropping_bomb_result.is_ok(),
        "Dropping the first bomb should be OK"
    );
    assert_eq!(
        game_state.board.get_cell(&position),
        Cell::Bomb([Some(0), None])
    );

    // Drop the second bomb
    let dropping_bomb_result = Game::drop_bomb(&mut game_state, position, ALICE);
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
    let game_state = Game::new_game(ALICE, BOB);
    assert_eq!(game_state.phase, GamePhase::Bomb);
    let game_state = Game::change_game_phase(game_state, GamePhase::Play);
    assert_eq!(game_state.phase, GamePhase::Play);
    let game_state = Game::change_game_phase(game_state, GamePhase::Bomb);
    assert_eq!(game_state.phase, GamePhase::Bomb);
}

#[test]
fn a_player_cannot_drop_a_stone_out_of_turn() {
    let state = Game::new_game(ALICE, BOB);
    let drop_stone_result = Game::drop_stone(state, BOB, Side::North, 0);
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

    let mut state = Game::new_game(ALICE, BOB);
    state.board.cells = cells;

    let state = Game::drop_stone(state, ALICE, Side::North, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 0 }),
        Cell::Stone(ALICE)
    );
    let state = Game::drop_stone(state, BOB, Side::North, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 8, col: 1 }),
        Cell::Stone(BOB)
    );
    let state = Game::drop_stone(state, ALICE, Side::North, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 2 }),
        Cell::Stone(ALICE)
    );
    assert_eq!(
        Game::drop_stone(state, BOB, Side::North, 3).unwrap_err(),
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

    let mut state = Game::new_game(ALICE, BOB);
    state.board.cells = cells;

    let state = Game::drop_stone(state, ALICE, Side::South, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 0 }),
        Cell::Stone(ALICE)
    );
    let state = Game::drop_stone(state, BOB, Side::South, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 1, col: 1 }),
        Cell::Stone(BOB)
    );
    let state = Game::drop_stone(state, ALICE, Side::South, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 2 }),
        Cell::Stone(ALICE)
    );
    assert_eq!(
        Game::drop_stone(state, BOB, Side::South, 3).unwrap_err(),
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

    let mut state = Game::new_game(ALICE, BOB);
    state.board.cells = cells;

    let state = Game::drop_stone(state, ALICE, Side::East, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 0 }),
        Cell::Stone(ALICE)
    );
    let state = Game::drop_stone(state, BOB, Side::East, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 1, col: 1 }),
        Cell::Stone(BOB)
    );
    let state = Game::drop_stone(state, ALICE, Side::East, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 2, col: 9 }),
        Cell::Stone(ALICE)
    );
    assert_eq!(
        Game::drop_stone(state, BOB, Side::East, 3).unwrap_err(),
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

    let mut state = Game::new_game(ALICE, BOB);
    state.board.cells = cells;

    let state = Game::drop_stone(state, ALICE, Side::West, 0).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 9 }),
        Cell::Stone(ALICE)
    );
    let state = Game::drop_stone(state, BOB, Side::West, 1).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 1, col: 8 }),
        Cell::Stone(BOB)
    );
    let state = Game::drop_stone(state, ALICE, Side::West, 2).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 2, col: 0 }),
        Cell::Stone(ALICE)
    );
    assert_eq!(
        Game::drop_stone(state, BOB, Side::West, 3).unwrap_err(),
        GameError::InvalidDroppingPosition
    );
}

#[test]
fn a_stone_should_explode_a_bomb_when_passing_through() {
    let o = Cell::Empty;
    let b = Cell::Bomb([Some(0), Some(1)]);
    let x = Cell::Stone(0);
    let l = Cell::Block;

    let mut state = Game::new_game(ALICE, BOB);
    state.board.cells = [
        [o, o, o, o, o, o, o, o, b, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, b, o, o, o, o, o, o],
        [o, o, o, o, x, b, l, o, o, o],
        [o, o, o, o, b, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, b],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, b, o, o, o, o, o],
    ];

    let dropping_stone_result = Game::drop_stone(state.clone(), ALICE, Side::North, 5);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();

    // Stone in position 3,4 should be destroyed.
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 3, col: 4 }),
        Cell::Empty
    );

    // Bomb in position 4,4 should be destroyed.
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 4, col: 4 }),
        Cell::Empty
    );

    // Block in position 3,6 should not be destroyed.
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 3, col: 6 }),
        Cell::Block
    );

    // Bomb in position 2,3 should not be destroyed.
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 2, col: 3 }),
        Cell::Bomb([Some(0), Some(1)])
    );

    let dropping_stone_result = Game::drop_stone(state.clone(), BOB, Side::North, 8);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 8 }),
        Cell::Empty
    );

    let dropping_stone_result = Game::drop_stone(state.clone(), ALICE, Side::East, 7);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 7, col: 9 }),
        Cell::Empty
    );

    let dropping_stone_result = Game::drop_stone(state.clone(), BOB, Side::South, 4);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 4 }),
        Cell::Empty
    );

    let dropping_stone_result = Game::drop_stone(state.clone(), ALICE, Side::South, 4);
    assert!(dropping_stone_result.is_ok());
    state = dropping_stone_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 4, col: 4 }),
        Cell::Empty
    );
}

#[test]
fn a_player_wins_when_has_a_four_stone_vertical_row() {
    let o = Cell::Empty;
    let s = Cell::Stone(ALICE);

    let mut state = Game::new_game(ALICE, BOB);
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
    assert_eq!(state.winner.unwrap(), ALICE);
}

#[test]
fn a_player_wins_when_has_a_four_stone_horizontal_row() {
    let o = Cell::Empty;
    let s = Cell::Stone(ALICE);

    let mut state = Game::new_game(ALICE, BOB);
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
    assert_eq!(state.winner.unwrap(), ALICE);
}

#[test]
fn a_player_wins_when_has_a_four_stone_ascending_diagonal_row() {
    let o = Cell::Empty;
    let s = Cell::Stone(ALICE);

    let mut state = Game::new_game(ALICE, BOB);
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
    assert_eq!(state.winner.unwrap(), ALICE);
}

#[test]
fn a_player_wins_when_has_a_four_stone_descending_diagonal_row() {
    let o = Cell::Empty;
    let s = Cell::Stone(ALICE);

    let mut state = Game::new_game(ALICE, BOB);
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
    assert_eq!(state.winner.unwrap(), ALICE);
}

#[test]
fn no_player_wins_for_less_than_four_in_a_row_stones() {
    let o = Cell::Empty;
    let b = Cell::Block;
    let r = Cell::Stone(ALICE);
    let m = Cell::Stone(BOB);

    let mut state = Game::new_game(ALICE, BOB);
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

#[test]
fn should_play_a_game() {
    let o = Cell::Empty;
    let b = Cell::Block;

    let mut state = Game::new_game(ALICE, BOB);
    state.board.cells = [
        [o, o, o, o, o, o, o, o, b, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, b, b, b, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [b, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, b, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
        [o, o, o, o, o, o, o, o, o, o],
    ];

    // players1 drops bombs
    let player1_num_bombs = state.bombs[&ALICE];
    let result = Game::drop_bomb(&mut state, Coordinates { row: 0, col: 0 }, ALICE);
    assert!(result.is_ok());
    assert_eq!(state.bombs[&ALICE], player1_num_bombs - 1);

    let dropping_result = Game::drop_bomb(&mut state, Coordinates { row: 0, col: 0 }, ALICE);
    assert!(
        dropping_result.is_err(),
        "Player cannot drop two bombs in the same position"
    );
    assert_eq!(state.bombs[&ALICE], player1_num_bombs - 1);

    let result = Game::drop_bomb(&mut state, Coordinates { row: 9, col: 9 }, ALICE);
    assert!(result.is_ok());
    assert_eq!(state.bombs[&ALICE], player1_num_bombs - 2);

    let result = Game::drop_bomb(&mut state, Coordinates { row: 7, col: 7 }, ALICE);
    assert!(result.is_ok());
    assert_eq!(state.bombs[&ALICE], player1_num_bombs - 3);

    let result = Game::drop_bomb(&mut state, Coordinates { row: 6, col: 8 }, ALICE);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        GameError::NoMoreBombsAvailable,
        "The player shouldn't have more bombs for dropping"
    );

    // players2 drops bombs
    let player2_num_bombs = state.bombs[&BOB];
    let result = Game::drop_bomb(&mut state, Coordinates { row: 9, col: 0 }, BOB);
    assert!(result.is_ok());
    assert_eq!(state.bombs[&BOB], player2_num_bombs - 1);

    let result = Game::drop_bomb(&mut state, Coordinates { row: 9, col: 9 }, BOB);
    assert!(
        result.is_ok(),
        "A cell should hold bombs of different players"
    );
    assert_eq!(state.bombs[&BOB], player2_num_bombs - 2);

    let result = Game::drop_bomb(&mut state, Coordinates { row: 9, col: 3 }, BOB);
    assert!(result.is_ok());
    assert_eq!(state.bombs[&BOB], player2_num_bombs - 3);

    assert_eq!(
        state.phase,
        GamePhase::Play,
        "The game should be in play phase after all bombs have been deployed"
    );

    let dropping_result = Game::drop_stone(state.clone(), BOB, Side::North, 0);
    assert!(dropping_result.is_err());
    assert_eq!(dropping_result.unwrap_err(), GameError::NotPlayerTurn);

    let dropping_result = Game::drop_stone(state.clone(), ALICE, Side::North, 0);
    assert!(dropping_result.is_ok());
    state = dropping_result.unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 0, col: 0 }),
        Cell::Empty
    );

    state = Game::drop_stone(state.clone(), BOB, Side::North, 2).unwrap();
    state = Game::drop_stone(state.clone(), ALICE, Side::South, 8).unwrap();
    state = Game::drop_stone(state.clone(), BOB, Side::North, 2).unwrap();
    state = Game::drop_stone(state.clone(), ALICE, Side::South, 8).unwrap();
    state = Game::drop_stone(state.clone(), BOB, Side::North, 2).unwrap();

    // player 1 explodes bomb on 9,3 and player 2 loses stones on 9,2 and 8,2
    state = Game::drop_stone(state.clone(), ALICE, Side::North, 3).unwrap();
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 9, col: 2 }),
        Cell::Empty
    );
    assert_eq!(
        state.board.get_cell(&Coordinates { row: 8, col: 2 }),
        Cell::Empty
    );

    state = Game::drop_stone(state.clone(), BOB, Side::North, 2).unwrap();
    state = Game::drop_stone(state.clone(), ALICE, Side::South, 8).unwrap();
    state = Game::drop_stone(state.clone(), BOB, Side::North, 2).unwrap();

    // No player has won yet.
    assert!(state.winner.is_none());
    state = Game::drop_stone(state.clone(), ALICE, Side::South, 8).unwrap();

    assert!(state.winner.is_some());
    assert_eq!(state.winner.unwrap(), ALICE);
}
