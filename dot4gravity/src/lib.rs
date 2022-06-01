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

use rand::Rng;

#[cfg(test)]
mod tests;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 10;
const NUM_OF_PLAYERS: usize = 2;
const NUM_OF_BOMBS_PER_PLAYER: usize = 3;
const NUM_OF_BLOCKS: usize = 10;

type Player = u32;

/// Represents a cell of the board.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Cell {
    Empty,
    Bomb([Option<Player>; NUM_OF_PLAYERS]),
    Block,
    Stone(Player),
}

impl Cell {
    /// Tells if a cell is suitable for dropping a bomb.
    fn is_valid_for_dropping_bomb(&self) -> bool {
        match self {
            Cell::Empty => true,
            Cell::Bomb([Some(_), None]) => true,
            _ => false,
        }
    }

    /// Tells if a stone can traverse the cell.
    fn is_travesable(&self) -> bool {
        *self == Cell::Empty
    }

    /// Tells if a cell is of type 'bomb'
    fn is_bomb(&self) -> bool {
        match self {
            Cell::Bomb([Some(_), None]) => true,
            Cell::Bomb([Some(_), Some(_)]) => true,
            _ => false,
        }
    }

    /// Tells if a cell must be cleared when it's affected by an explosion.
    fn is_explodable(&self) -> bool {
        *self != Cell::Block
    }
}

/// Coordinates for a cell in the board.
#[derive(Clone, Copy, PartialEq)]
struct Coordinates {
    row: usize,
    col: usize,
}

impl Coordinates {
    fn is_inside_board(&self) -> bool {
        self.row < BOARD_WIDTH && self.col < BOARD_HEIGHT
    }

    fn is_edge_position(&self) -> bool {
        self.col == BOARD_WIDTH - 1
            || self.row == BOARD_HEIGHT - 1
            || self.col == 0
            || self.row == 0
    }
}

/// Sides of the board from which a player can drop a stone.
enum Side {
    North,
    East,
    South,
    West,
}

#[derive(Debug)]
struct Board {
    cells: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
    pub fn new() -> Board {
        Board {
            cells: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }

    fn get_cell(&self, position: &Coordinates) -> Cell {
        self.cells[position.row][position.col]
    }

    fn change_cell(&mut self, position: Coordinates, cell: Cell) {
        self.cells[position.row][position.col] = cell;
        assert_eq!(self.cells[position.row][position.col], cell);
    }

    fn populate_with_random_blocks(mut board: Board, mut num_of_blocks: usize) -> Board {
        let mut blocks = Vec::<Coordinates>::new();
        let mut rng = rand::thread_rng();

        while num_of_blocks > 0 {
            let block_coordinates = Coordinates {
                row: rng.gen_range(0..BOARD_HEIGHT),
                col: rng.gen_range(0..BOARD_WIDTH),
            };
            if !blocks.contains(&block_coordinates) {
                num_of_blocks -= 1;
                blocks.push(block_coordinates)
            }
        }
        for block_coordinates in blocks {
            board.change_cell(block_coordinates.clone(), Cell::Block);
        }
        board
    }

    fn explode_bomb(mut board: Board, bomb_position: Coordinates) -> Board {
        let offsets: [(i8, i8); 9] = [
            (0, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
        ];

        // Collect the explodable cells around.
        let explodable_cells_coords: Vec<Coordinates> = offsets
            .iter()
            .map(|offset| {
                (
                    offset.0 + bomb_position.row as i8,
                    offset.1 + bomb_position.col as i8,
                )
            })
            .filter(|offset| offset.0 >= 0 && offset.1 >= 0)
            .map(|offset| Coordinates {
                row: offset.0 as usize,
                col: offset.1 as usize,
            })
            .filter(|position| {
                position.is_inside_board() && board.get_cell(&position).is_explodable()
            })
            .collect();

        for position in explodable_cells_coords {
            board.change_cell(position, Cell::Empty);
        }

        board
    }
}

#[derive(Debug, Eq, PartialEq)]
enum GamePhase {
    /// Not turn based. The players place bombs during this phase.
    Bomb,
    /// Turn based phase. Every player can trigger bombs, his own or opponents.
    Play,
}

#[derive(Debug, Eq, PartialEq)]
enum GameError {
    /// Tried to drop a bomb during game play phase.
    DroppedBombDuringPlayPhase,
    /// The player has no more bombs to drop.
    NoMoreBombsAvailable,
    /// Tried to drop a bomb in an invalid cell. The cell is already taken.
    InvalidBombPosition,
    /// Tried to drop in an invalid position.
    InvalidDroppingPosition,
    /// Tried to drop a stone during other player's turn
    NotPlayerTurn,
}

#[derive(Debug)]
struct GameState {
    /// Represents the game board.
    board: Board,
    /// Game mode.
    phase: GamePhase,
    /// When present,it contains the player that won.
    winner: Option<Player>,
    /// Next player turn.
    next_player: Player,
    /// Number of bombs available for each player.
    bombs: [usize; NUM_OF_PLAYERS],
}

struct Game;

impl Game {
    /// Create a new game.
    fn new_game() -> GameState {
        GameState {
            board: Board::populate_with_random_blocks(Board::new(), NUM_OF_BLOCKS),
            phase: GamePhase::Bomb,
            winner: None,
            next_player: 0,
            bombs: [NUM_OF_BOMBS_PER_PLAYER; NUM_OF_PLAYERS],
        }
    }

    /// Drop a bomb. Called during bomb phase.
    fn drop_bomb(
        mut game_state: GameState,
        position: Coordinates,
        player: Player,
    ) -> Result<GameState, GameError> {
        if game_state.phase == GamePhase::Play {
            return Err(GameError::DroppedBombDuringPlayPhase);
        }
        if game_state.bombs[player as usize] == 0 {
            return Err(GameError::NoMoreBombsAvailable);
        }
        if !game_state
            .board
            .get_cell(&position)
            .is_valid_for_dropping_bomb()
        {
            return Err(GameError::InvalidBombPosition);
        }

        game_state.bombs[player as usize] -= 1;
        match game_state.board.cells[position.row][position.col] {
            Cell::Empty => {
                game_state.board.cells[position.row][position.col] =
                    Cell::Bomb([Some(player), None]);
            }
            Cell::Bomb([Some(other_player), None]) => {
                if other_player == player {
                    return Err(GameError::InvalidBombPosition);
                } else {
                    game_state.board.cells[position.row][position.col] =
                        Cell::Bomb([Some(other_player), Some(player)]);
                }
            }
            Cell::Bomb([Some(_), Some(_)]) => {
                return Err(GameError::InvalidBombPosition);
            }
            _ => return Err(GameError::InvalidBombPosition),
        }

        Ok(game_state)
    }

    /// Change game phase.
    fn change_game_phase(mut game_state: GameState, phase: GamePhase) -> GameState {
        game_state.phase = phase;
        game_state
    }

    /// Drop stone. Called during play phase.
    fn drop_stone(
        mut game_state: GameState,
        player: Player,
        side: Side,
        position: usize,
    ) -> Result<GameState, GameError> {
        if position >= BOARD_HEIGHT || position >= BOARD_WIDTH {
            return Err(GameError::InvalidDroppingPosition);
        }

        if game_state.next_player != player {
            return Err(GameError::NotPlayerTurn);
        }

        let is_travesable_cell = |position: &Coordinates| -> bool {
            game_state.board.get_cell(position).is_travesable()
        };

        fn update_board_for_stone(
            mut game_state: GameState,
            stone_position: &Coordinates,
            stone_prev_position: &Coordinates,
            player: Player,
        ) -> GameState {
            match game_state.board.get_cell(&stone_position) {
                // A cell bomb must explode.
                Cell::Bomb([_, _]) => {
                    game_state.board = Board::explode_bomb(game_state.board, *stone_position);
                }
                // The stone is placed at the end if it's empty.
                Cell::Empty => {
                    if stone_position.is_edge_position() {
                        game_state
                            .board
                            .change_cell(*stone_position, Cell::Stone(player));
                    }
                }
                // The stone is placed in the previous position of a block.
                Cell::Block => {
                    game_state
                        .board
                        .change_cell(*stone_prev_position, Cell::Stone(player));
                }
                // The stone is placed in the previous position of a stone.
                Cell::Stone(_) => {
                    game_state
                        .board
                        .change_cell(*stone_prev_position, Cell::Stone(player));
                }
            }

            game_state
        }

        match side {
            Side::North => {
                if !is_travesable_cell(&Coordinates {
                    row: 0,
                    col: position,
                }) {
                    return Err(GameError::InvalidDroppingPosition);
                }

                for row in 1..BOARD_HEIGHT {
                    game_state = update_board_for_stone(
                        game_state,
                        &Coordinates { row, col: position },
                        &Coordinates {
                            row: row - 1,
                            col: position,
                        },
                        player,
                    );
                }
            }
            Side::East => {
                if !is_travesable_cell(&Coordinates {
                    row: position,
                    col: BOARD_WIDTH - 1,
                }) {
                    return Err(GameError::InvalidDroppingPosition);
                }

                for col in (0..BOARD_WIDTH - 1).rev() {
                    game_state = update_board_for_stone(
                        game_state,
                        &Coordinates {
                            row: position,
                            col: col,
                        },
                        &Coordinates {
                            row: position,
                            col: col + 1,
                        },
                        player,
                    );
                }
            }
            Side::South => {
                if !is_travesable_cell(&Coordinates {
                    row: BOARD_HEIGHT - 1,
                    col: position,
                }) {
                    return Err(GameError::InvalidDroppingPosition);
                }

                for row in (0..BOARD_HEIGHT - 1).rev() {
                    game_state = update_board_for_stone(
                        game_state,
                        &Coordinates { row, col: position },
                        &Coordinates {
                            row: row + 1,
                            col: position,
                        },
                        player,
                    );
                }
            }
            Side::West => {
                if !is_travesable_cell(&Coordinates {
                    row: position,
                    col: 0,
                }) {
                    return Err(GameError::InvalidDroppingPosition);
                }

                for col in 1..BOARD_WIDTH {
                    game_state = update_board_for_stone(
                        game_state,
                        &Coordinates {
                            row: position,
                            col: col,
                        },
                        &Coordinates {
                            row: position,
                            col: col - 1,
                        },
                        player,
                    );
                }
            }
        }

        Ok(game_state)
    }

    fn check_winner_player(mut game_state: GameState) -> GameState {
        if game_state.winner.is_some() {
            return game_state;
        }

        let board = &game_state.board;
        // Check vertical
        for row in 0..BOARD_HEIGHT - 3 {
            for col in 0..BOARD_WIDTH {
                let cell = board.get_cell(&Coordinates { row, col });
                match cell {
                    Cell::Stone(player) => {
                        if cell == board.get_cell(&Coordinates { row: row + 1, col })
                            && cell == board.get_cell(&Coordinates { row: row + 2, col })
                            && cell == board.get_cell(&Coordinates { row: row + 3, col })
                        {
                            game_state.winner = Some(player);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check horizontal
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH - 3 {
                let cell = board.get_cell(&Coordinates { row, col });
                match cell {
                    Cell::Stone(player) => {
                        if cell == board.get_cell(&Coordinates { row, col: col + 1 })
                            && cell == board.get_cell(&Coordinates { row, col: col + 2 })
                            && cell == board.get_cell(&Coordinates { row, col: col + 3 })
                        {
                            game_state.winner = Some(player);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check ascending diagonal
        for row in 3..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH - 3 {
                let cell = board.get_cell(&Coordinates { row, col });
                match cell {
                    Cell::Stone(player) => {
                        if cell
                            == board.get_cell(&Coordinates {
                                row: row - 1,
                                col: col + 1,
                            })
                            && cell
                                == board.get_cell(&Coordinates {
                                    row: row - 2,
                                    col: col + 2,
                                })
                            && cell
                                == board.get_cell(&Coordinates {
                                    row: row - 3,
                                    col: col + 3,
                                })
                        {
                            game_state.winner = Some(player);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check diagonal descending
        for row in 0..BOARD_HEIGHT - 3 {
            for col in 0..BOARD_WIDTH - 3 {
                let cell = board.get_cell(&Coordinates { row, col });
                match cell {
                    Cell::Stone(player) => {
                        if cell
                            == board.get_cell(&Coordinates {
                                row: row + 1,
                                col: col + 1,
                            })
                            && cell
                                == board.get_cell(&Coordinates {
                                    row: row + 2,
                                    col: col + 2,
                                })
                            && cell
                                == board.get_cell(&Coordinates {
                                    row: row + 3,
                                    col: col + 3,
                                })
                        {
                            game_state.winner = Some(player);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        game_state
    }
}
