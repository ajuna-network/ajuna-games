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

use std::collections::HashMap;

use rand::prelude::SliceRandom;

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
        *self == Cell::Empty || self.is_bomb()
    }

    /// Tells if a cell is of type 'bomb'
    fn is_bomb(&self) -> bool {
        matches!(self, Cell::Bomb(_))
    }

    /// Tells if a cell must be cleared when it's affected by an explosion.
    fn is_explodable(&self) -> bool {
        *self != Cell::Block
    }
}

/// Coordinates for a cell in the board.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coordinates {
    row: usize,
    col: usize,
}

impl Coordinates {
    /// Tells if a cell is inside the board.
    fn is_inside_board(&self) -> bool {
        self.row < BOARD_WIDTH && self.col < BOARD_HEIGHT
    }

    /// Tells if a cell is in the opposite of a side.
    fn is_opposite_cell(&self, side: Side) -> bool {
        match side {
            Side::North => self.row == BOARD_HEIGHT - 1,
            Side::East => self.col == 0,
            Side::South => self.row == 0,
            Side::West => self.col == BOARD_WIDTH - 1,
        }
    }
}

/// Sides of the board from which a player can drop a stone.
#[derive(Clone, Debug)]
pub enum Side {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    cells: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Default for Board {
    fn default() -> Self {
        Board {
            cells: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }
}

impl Board {
    pub fn new() -> Board {
        Board::default()
    }

    fn get_cell(&self, position: &Coordinates) -> Cell {
        self.cells[position.row][position.col]
    }

    fn update_cell(&mut self, position: Coordinates, cell: Cell) {
        self.cells[position.row][position.col] = cell;
        assert_eq!(self.cells[position.row][position.col], cell);
    }

    fn populate_with_random_blocks(mut board: Board, num_of_blocks: usize) -> Board {
        let mut rng = rand::thread_rng();

        let mut board_coordinates = Vec::new();
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_HEIGHT {
                board_coordinates.push(Coordinates { row, col });
            }
        }
        board_coordinates
            .choose_multiple(&mut rng, num_of_blocks)
            .cloned()
            .for_each(|coordinates| board.update_cell(coordinates, Cell::Block));

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
        offsets
            .iter()
            .map(|(row_offset, col_offset)| Coordinates {
                row: (row_offset + bomb_position.row as i8) as usize,
                col: (col_offset + bomb_position.col as i8) as usize,
            })
            .for_each(|position| {
                if position.is_inside_board() && board.get_cell(&position).is_explodable() {
                    board.update_cell(position, Cell::Empty)
                }
            });

        board
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GamePhase {
    /// Not turn based. The players place bombs during this phase.
    Bomb,
    /// Turn based phase. Every player can trigger bombs, his own or opponents.
    Play,
}

#[derive(Debug, Eq, PartialEq)]
pub enum GameError {
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
    /// The cell has no previous position. It is an edge cell.
    NoPreviousPosition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    /// Represents the game board.
    pub board: Board,
    /// Game mode.
    pub phase: GamePhase,
    /// When present,it contains the player that won.
    pub winner: Option<Player>,
    /// Next player turn.
    pub next_player: Player,
    /// Number of bombs available for each player.
    pub bombs: HashMap<Player, usize>,
}

impl GameState {
    fn is_all_bomb_dropped(&self) -> bool {
        self.bombs.iter().all(|bombs| *bombs.1 == 0usize)
    }

    fn change_game_phase(&mut self, phase: GamePhase) {
        self.phase = phase
    }
}

pub struct Game;

impl Game {
    /// Create a new game.
    pub fn new_game(player1: Player, player2: Player) -> GameState {
        GameState {
            board: Board::populate_with_random_blocks(Board::new(), NUM_OF_BLOCKS),
            phase: GamePhase::Bomb,
            winner: None,
            next_player: player1,
            bombs: HashMap::from([
                (player1, NUM_OF_BOMBS_PER_PLAYER),
                (player2, NUM_OF_BOMBS_PER_PLAYER),
            ]),
        }
    }

    /// Drop a bomb. Called during bomb phase.
    pub fn drop_bomb(
        game_state: &mut GameState,
        position: Coordinates,
        player: Player,
    ) -> Result<bool, GameError> {
        if game_state.phase == GamePhase::Play {
            return Err(GameError::DroppedBombDuringPlayPhase);
        }
        if game_state.bombs[&player] == 0 {
            return Err(GameError::NoMoreBombsAvailable);
        }
        if !game_state
            .board
            .get_cell(&position)
            .is_valid_for_dropping_bomb()
        {
            return Err(GameError::InvalidBombPosition);
        }
        match game_state.board.cells[position.row][position.col] {
            Cell::Empty => {
                game_state
                    .board
                    .update_cell(position, Cell::Bomb([Some(player), None]));
                *game_state.bombs.get_mut(&player).unwrap() -= 1;
                if game_state.is_all_bomb_dropped() {
                    game_state.change_game_phase(GamePhase::Play);
                }
            }
            Cell::Bomb([Some(other_player), None]) => {
                if other_player == player {
                    return Err(GameError::InvalidBombPosition);
                } else {
                    game_state
                        .board
                        .update_cell(position, Cell::Bomb([Some(other_player), Some(player)]));
                    *game_state.bombs.get_mut(&player).unwrap() -= 1;
                    if game_state.is_all_bomb_dropped() {
                        game_state.change_game_phase(GamePhase::Play);
                    }
                }
            }
            Cell::Bomb([Some(_), Some(_)]) => {
                return Err(GameError::InvalidBombPosition);
            }
            _ => return Err(GameError::InvalidBombPosition),
        }

        Ok(true)
    }

    /// Change game phase.
    pub fn change_game_phase(mut game_state: GameState, phase: GamePhase) -> GameState {
        game_state.change_game_phase(phase);
        game_state
    }

    /// Drop stone. Called during play phase.
    pub fn drop_stone(
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

        match side {
            Side::North => {
                let mut row = 0;
                let mut stop = false;
                while row < BOARD_HEIGHT && !stop {
                    let position = Coordinates { row, col: position };
                    match game_state.board.get_cell(&position) {
                        // A cell bomb must explode.
                        Cell::Bomb([_, _]) => {
                            game_state.board = Board::explode_bomb(game_state.board, position);
                            stop = true;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side.clone()) {
                                game_state.board.update_cell(position, Cell::Stone(player));
                                stop = true;
                            }
                        }
                        // The stone is placed in the position previous to a block.
                        Cell::Block => {
                            if row > 0 {
                                game_state.board.update_cell(
                                    Coordinates {
                                        row: position.row - 1,
                                        col: position.col,
                                    },
                                    Cell::Stone(player),
                                );
                            } else {
                                return Err(GameError::InvalidDroppingPosition);
                            }
                            stop = true;
                        }
                        // The stone is placed in the previous position of a stone.
                        Cell::Stone(_) => {
                            if row > 0 {
                                game_state.board.update_cell(
                                    Coordinates {
                                        row: position.row - 1,
                                        col: position.col,
                                    },
                                    Cell::Stone(player),
                                );
                            } else {
                                return Err(GameError::InvalidDroppingPosition);
                            }
                            stop = true;
                        }
                    }
                    row += 1;
                }
            }
            Side::East => {
                let mut col = BOARD_WIDTH - 1;

                loop {
                    let position = Coordinates { row: position, col };
                    match game_state.board.get_cell(&position) {
                        // A cell bomb must explode.
                        Cell::Bomb([_, _]) => {
                            game_state.board = Board::explode_bomb(game_state.board, position);
                            break;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side.clone()) {
                                game_state.board.update_cell(position, Cell::Stone(player));
                                break;
                            }
                        }
                        // The stone is placed in the position previous to a block.
                        Cell::Block => {
                            if col < BOARD_WIDTH - 1 {
                                game_state.board.update_cell(
                                    Coordinates {
                                        row: position.row,
                                        col: position.col + 1,
                                    },
                                    Cell::Stone(player),
                                );
                            } else {
                                return Err(GameError::InvalidDroppingPosition);
                            }
                            break;
                        }
                        // The stone is placed in the previous position of a stone.
                        Cell::Stone(_) => {
                            if col < BOARD_WIDTH - 1 {
                                game_state.board.update_cell(
                                    Coordinates {
                                        row: position.row,
                                        col: position.col + 1,
                                    },
                                    Cell::Stone(player),
                                );
                            } else {
                                return Err(GameError::InvalidDroppingPosition);
                            }
                            break;
                        }
                    }
                    if col == 0 {
                        break;
                    };
                    col -= 1;
                }
            }
            Side::South => {
                let mut row = BOARD_HEIGHT - 1;

                loop {
                    let position = Coordinates { row, col: position };
                    match game_state.board.get_cell(&position) {
                        // A cell bomb must explode.
                        Cell::Bomb([_, _]) => {
                            game_state.board = Board::explode_bomb(game_state.board, position);
                            break;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side.clone()) {
                                game_state.board.update_cell(position, Cell::Stone(player));
                                break;
                            }
                        }
                        // The stone is placed in the position previous to a block.
                        Cell::Block => {
                            if row < BOARD_HEIGHT - 1 {
                                game_state.board.update_cell(
                                    Coordinates {
                                        row: position.row + 1,
                                        col: position.col,
                                    },
                                    Cell::Stone(player),
                                );
                            } else {
                                return Err(GameError::InvalidDroppingPosition);
                            }
                            break;
                        }
                        // The stone is placed in the previous position of a stone.
                        Cell::Stone(_) => {
                            if row < BOARD_HEIGHT - 1 {
                                game_state.board.update_cell(
                                    Coordinates {
                                        row: position.row + 1,
                                        col: position.col,
                                    },
                                    Cell::Stone(player),
                                );
                            } else {
                                return Err(GameError::InvalidDroppingPosition);
                            }
                            break;
                        }
                    }

                    if row == 0 {
                        break;
                    }
                    row -= 1;
                }
            }
            Side::West => {
                let mut col = 0;
                let mut stop = false;
                while col < BOARD_WIDTH && !stop {
                    let position = Coordinates { row: position, col };
                    match game_state.board.get_cell(&position) {
                        // A cell bomb must explode.
                        Cell::Bomb([_, _]) => {
                            game_state.board = Board::explode_bomb(game_state.board, position);
                            stop = true;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side.clone()) {
                                game_state.board.update_cell(position, Cell::Stone(player));
                                stop = true;
                            }
                        }
                        // The stone is placed in the position previous to a block.
                        Cell::Block => {
                            if col > 0 {
                                game_state.board.update_cell(
                                    Coordinates {
                                        row: position.row,
                                        col: position.col - 1,
                                    },
                                    Cell::Stone(player),
                                );
                            } else {
                                return Err(GameError::InvalidDroppingPosition);
                            }
                            stop = true;
                        }
                        // The stone is placed in the previous position of a stone.
                        Cell::Stone(_) => {
                            if col < BOARD_WIDTH - 1 {
                                game_state.board.update_cell(
                                    Coordinates {
                                        row: position.row,
                                        col: position.col - 1,
                                    },
                                    Cell::Stone(player),
                                );
                            } else {
                                return Err(GameError::InvalidDroppingPosition);
                            }
                            stop = true;
                        }
                    }
                    col += 1;
                }
            }
        }

        game_state.next_player = (game_state.next_player + 1) % NUM_OF_PLAYERS as u32;
        game_state = Game::check_winner_player(game_state);

        Ok(game_state)
    }

    pub fn check_winner_player(mut game_state: GameState) -> GameState {
        if game_state.winner.is_some() {
            return game_state;
        }

        let board = &game_state.board;
        // Check vertical
        for row in 0..BOARD_HEIGHT - 3 {
            for col in 0..BOARD_WIDTH {
                let cell = board.get_cell(&Coordinates { row, col });
                if let Cell::Stone(player) = cell {
                    if cell == board.get_cell(&Coordinates { row: row + 1, col })
                        && cell == board.get_cell(&Coordinates { row: row + 2, col })
                        && cell == board.get_cell(&Coordinates { row: row + 3, col })
                    {
                        game_state.winner = Some(player);
                        break;
                    }
                }
            }
        }

        // Check horizontal
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH - 3 {
                let cell = board.get_cell(&Coordinates { row, col });
                if let Cell::Stone(player) = cell {
                    if cell == board.get_cell(&Coordinates { row, col: col + 1 })
                        && cell == board.get_cell(&Coordinates { row, col: col + 2 })
                        && cell == board.get_cell(&Coordinates { row, col: col + 3 })
                    {
                        game_state.winner = Some(player);
                        break;
                    }
                }
            }
        }

        // Check ascending diagonal
        for row in 3..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH - 3 {
                let cell = board.get_cell(&Coordinates { row, col });
                if let Cell::Stone(player) = cell {
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
            }
        }

        // Check diagonal descending
        for row in 0..BOARD_HEIGHT - 3 {
            for col in 0..BOARD_WIDTH - 3 {
                let cell = board.get_cell(&Coordinates { row, col });
                if let Cell::Stone(player) = cell {
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
            }
        }

        game_state
    }
}
