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

use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use rand::prelude::SliceRandom;
use scale_info::TypeInfo;

#[cfg(test)]
mod tests;

const BOARD_WIDTH: u8 = 10;
const BOARD_HEIGHT: u8 = 10;
const NUM_OF_PLAYERS: usize = 2;
const NUM_OF_BOMBS_PER_PLAYER: u8 = 3;
const NUM_OF_BLOCKS: usize = 10;

type PlayerIndex = u8;

/// Represents a cell of the board.
#[allow(private_in_public)]
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Copy, Debug, Eq, PartialEq)]
enum Cell {
    Empty,
    Bomb([Option<PlayerIndex>; NUM_OF_PLAYERS]),
    Block,
    Stone(PlayerIndex),
}

impl Default for Cell {
    fn default() -> Self {
        Self::Empty
    }
}

impl Cell {
    /// Tells if a cell is suitable for dropping a bomb.
    fn is_bomb_droppable(&self) -> bool {
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
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Coordinates {
    pub row: u8,
    pub col: u8,
}

impl Coordinates {
    pub const fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }

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
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    North,
    East,
    South,
    West,
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Copy, Clone, Eq, Debug, Default, PartialEq)]
pub struct Board {
    cells: [[Cell; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
}

impl Board {
    pub fn new() -> Board {
        Board::default()
    }

    fn is_bomb_droppable(&self, position: &Coordinates) -> bool {
        position.is_inside_board() && self.get_cell(position).is_bomb_droppable()
    }

    fn is_explodable(&self, position: &Coordinates) -> bool {
        position.is_inside_board() && self.get_cell(position).is_explodable()
    }

    fn get_cell(&self, position: &Coordinates) -> Cell {
        let cell = &self.cells[position.row as usize][position.col as usize];
        *cell
    }

    fn update_cell(&mut self, position: Coordinates, cell: Cell) {
        self.cells[position.row as usize][position.col as usize] = cell;
        assert_eq!(
            self.cells[position.row as usize][position.col as usize],
            cell
        );
    }

    fn explode_bomb(&mut self, bomb_position: Coordinates) {
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
            .map(|(row_offset, col_offset)| {
                Coordinates::new(
                    (row_offset + bomb_position.row as i8) as u8,
                    (col_offset + bomb_position.col as i8) as u8,
                )
            })
            .for_each(|position| {
                if self.is_explodable(&position) {
                    self.update_cell(position, Cell::Empty)
                }
            });
    }
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Copy, Clone, Debug, Eq, PartialEq)]
pub enum GamePhase {
    /// Not turn based. The players place bombs during this phase.
    Bomb,
    /// Turn based phase. Every player can trigger bombs, his own or opponents.
    Play,
}

impl Default for GamePhase {
    fn default() -> Self {
        Self::Bomb
    }
}

#[derive(Encode, Decode, TypeInfo, Debug, Eq, PartialEq)]
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

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Copy, Clone, Debug, Eq, PartialEq)]
pub struct GameState<Player> {
    /// Represents the game board.
    pub board: Board,
    /// Game mode.
    pub phase: GamePhase,
    /// When present,it contains the player that won.
    pub winner: Option<PlayerIndex>,
    /// Next player turn.
    pub next_player: PlayerIndex,
    /// Players:
    pub players: [Player; NUM_OF_PLAYERS],
    /// Number of bombs available for each player.
    pub bombs: [(Player, u8); NUM_OF_PLAYERS],
}

impl<Player: PartialEq> GameState<Player> {
    fn is_all_bomb_dropped(&self) -> bool {
        self.bombs.iter().all(|(_, bombs)| *bombs == 0)
    }

    fn change_game_phase(&mut self, phase: GamePhase) {
        self.phase = phase
    }

    pub fn is_player_in_game(&self, player: &Player) -> bool {
        self.bombs.iter().any(|(p, _)| *p == *player)
    }

    pub fn is_all_player_bomb_dropped(&self, player: &Player) -> bool {
        matches!(self.get_player_bombs(player), Some(available_bombs) if available_bombs == 0)
    }

    pub fn get_player_bombs(&self, player: &Player) -> Option<u8> {
        self.bombs
            .iter()
            .find(|(p, _)| *p == *player)
            .map(|(_, available_bombs)| *available_bombs)
    }

    pub fn decrease_player_bombs(&mut self, player: &Player) {
        for (p, bombs) in self.bombs.iter_mut() {
            if *p == *player {
                *bombs -= 1;
            }
        }
    }
    pub fn is_player_turn(&self, player: &Player) -> bool {
        self.players[self.next_player as usize] == *player
    }
    fn player_index(&self, player: &Player) -> PlayerIndex {
        let player_index = self
            .players
            .iter()
            .position(|this_player| this_player == player)
            .expect("game to always start with 2 players") as u8;
        player_index
    }
}

pub trait BlocksGenerator {
    /// Add blocks to the board.
    fn add_blocks(board: Board, num_of_blocks: usize) -> Board;
}

#[derive(Encode, Decode, TypeInfo)]
pub struct RandomBlocksGenerator;

impl BlocksGenerator for RandomBlocksGenerator {
    /// Generates blocks randomly located.
    fn add_blocks(mut board: Board, num_of_blocks: usize) -> Board {
        let mut rng = rand::thread_rng();

        let mut board_coordinates = Vec::new();
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_HEIGHT {
                board_coordinates.push(Coordinates::new(row, col));
            }
        }
        board_coordinates
            .choose_multiple(&mut rng, num_of_blocks)
            .cloned()
            .for_each(|coordinates| board.update_cell(coordinates, Cell::Block));

        board
    }
}

#[derive(Encode, Decode, TypeInfo)]
pub struct Game<Player>(PhantomData<Player>);

impl<Player: PartialEq> Game<Player> {
    fn can_drop_bomb(
        game_state: &GameState<Player>,
        position: &Coordinates,
        player: &Player,
    ) -> Result<(), GameError> {
        if game_state.phase == GamePhase::Play {
            return Err(GameError::DroppedBombDuringPlayPhase);
        }
        if game_state.is_all_player_bomb_dropped(player) {
            return Err(GameError::NoMoreBombsAvailable);
        }
        if !game_state.board.is_bomb_droppable(position) {
            return Err(GameError::InvalidBombPosition);
        }
        Ok(())
    }

    fn can_drop_stone(
        game_state: &GameState<Player>,
        position: u8,
        player: &Player,
    ) -> Result<(), GameError> {
        if position >= BOARD_HEIGHT || position >= BOARD_WIDTH {
            return Err(GameError::InvalidDroppingPosition);
        }
        if !game_state.is_player_turn(player) {
            return Err(GameError::NotPlayerTurn);
        }
        Ok(())
    }
}

impl<Player: PartialEq + Clone> Game<Player> {
    /// Create a new game.
    pub fn new_game<R: BlocksGenerator>(player1: Player, player2: Player) -> GameState<Player> {
        GameState {
            board: R::add_blocks(Board::new(), NUM_OF_BLOCKS),
            phase: Default::default(),
            winner: Default::default(),
            next_player: Default::default(),
            players: [player1.clone(), player2.clone()],
            bombs: [
                (player1, NUM_OF_BOMBS_PER_PLAYER),
                (player2, NUM_OF_BOMBS_PER_PLAYER),
            ],
        }
    }

    /// Drop a bomb. Called during bomb phase.
    pub fn drop_bomb(
        mut game_state: GameState<Player>,
        position: Coordinates,
        player: Player,
    ) -> Result<GameState<Player>, GameError> {
        Self::can_drop_bomb(&game_state, &position, &player)?;
        let player_index = game_state.player_index(&player);
        match game_state.board.get_cell(&position) {
            Cell::Empty => {
                game_state
                    .board
                    .update_cell(position, Cell::Bomb([Some(player_index), None]));
                game_state.decrease_player_bombs(&player);
                if game_state.is_all_bomb_dropped() {
                    game_state.change_game_phase(GamePhase::Play);
                }
            }
            Cell::Bomb([Some(other_player_index), None]) => {
                if other_player_index != player_index {
                    game_state.board.update_cell(
                        position,
                        Cell::Bomb([Some(other_player_index), Some(player_index)]),
                    );
                    game_state.decrease_player_bombs(&player);
                    if game_state.is_all_bomb_dropped() {
                        game_state.change_game_phase(GamePhase::Play);
                    }
                } else {
                    return Err(GameError::InvalidBombPosition);
                }
            }
            _ => return Err(GameError::InvalidBombPosition),
        }

        Ok(game_state)
    }

    /// Change game phase.
    pub fn change_game_phase(
        mut game_state: GameState<Player>,
        phase: GamePhase,
    ) -> GameState<Player> {
        game_state.change_game_phase(phase);
        game_state
    }

    /// Drop stone. Called during play phase.
    pub fn drop_stone(
        mut game_state: GameState<Player>,
        player: Player,
        side: Side,
        position: u8,
    ) -> Result<GameState<Player>, GameError> {
        Self::can_drop_stone(&game_state, position, &player)?;
        let player_index = game_state.player_index(&player);
        match side {
            Side::North => {
                let mut row = 0;
                let mut stop = false;
                while row < BOARD_HEIGHT && !stop {
                    let position = Coordinates::new(row, position);
                    match game_state.board.get_cell(&position) {
                        // A cell bomb must explode.
                        Cell::Bomb([_, _]) => {
                            game_state.board.explode_bomb(position);
                            stop = true;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side.clone()) {
                                game_state
                                    .board
                                    .update_cell(position, Cell::Stone(player_index));
                                stop = true;
                            }
                        }
                        // The stone is placed in the position previous to a block.
                        Cell::Block => {
                            if row > 0 {
                                game_state.board.update_cell(
                                    Coordinates::new(position.row - 1, position.col),
                                    Cell::Stone(player_index),
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
                                    Coordinates::new(position.row - 1, position.col),
                                    Cell::Stone(player_index),
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
                    let position = Coordinates::new(position, col);
                    match game_state.board.get_cell(&position) {
                        // A cell bomb must explode.
                        Cell::Bomb([_, _]) => {
                            game_state.board.explode_bomb(position);
                            break;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side.clone()) {
                                game_state
                                    .board
                                    .update_cell(position, Cell::Stone(player_index));
                                break;
                            }
                        }
                        // The stone is placed in the position previous to a block.
                        Cell::Block => {
                            if col < BOARD_WIDTH - 1 {
                                game_state.board.update_cell(
                                    Coordinates::new(position.row, position.col + 1),
                                    Cell::Stone(player_index),
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
                                    Coordinates::new(position.row, position.col + 1),
                                    Cell::Stone(player_index),
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
                    let position = Coordinates::new(row, position);
                    match game_state.board.get_cell(&position) {
                        // A cell bomb must explode.
                        Cell::Bomb([_, _]) => {
                            game_state.board.explode_bomb(position);
                            break;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side.clone()) {
                                game_state
                                    .board
                                    .update_cell(position, Cell::Stone(player_index));
                                break;
                            }
                        }
                        // The stone is placed in the position previous to a block.
                        Cell::Block => {
                            if row < BOARD_HEIGHT - 1 {
                                game_state.board.update_cell(
                                    Coordinates::new(position.row + 1, position.col),
                                    Cell::Stone(player_index),
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
                                    Coordinates::new(position.row + 1, position.col),
                                    Cell::Stone(player_index),
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
                    let position = Coordinates::new(position, col);
                    match game_state.board.get_cell(&position) {
                        // A cell bomb must explode.
                        Cell::Bomb([_, _]) => {
                            game_state.board.explode_bomb(position);
                            stop = true;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side.clone()) {
                                game_state
                                    .board
                                    .update_cell(position, Cell::Stone(player_index));
                                stop = true;
                            }
                        }
                        // The stone is placed in the position previous to a block.
                        Cell::Block => {
                            if col > 0 {
                                game_state.board.update_cell(
                                    Coordinates::new(position.row, position.col - 1),
                                    Cell::Stone(player_index),
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
                                    Coordinates::new(position.row, position.col - 1),
                                    Cell::Stone(player_index),
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

        game_state.next_player = (game_state.next_player + 1) % NUM_OF_PLAYERS as u8;
        game_state = Game::check_winner_player(game_state);

        Ok(game_state)
    }

    pub fn check_winner_player(mut game_state: GameState<Player>) -> GameState<Player> {
        if game_state.winner.is_some() {
            return game_state;
        }

        let board = &game_state.board;
        // Check vertical
        for row in 0..BOARD_HEIGHT - 3 {
            for col in 0..BOARD_WIDTH {
                let cell = board.get_cell(&Coordinates::new(row, col));
                if let Cell::Stone(player_index) = cell {
                    if cell == board.get_cell(&Coordinates::new(row + 1, col))
                        && cell == board.get_cell(&Coordinates::new(row + 2, col))
                        && cell == board.get_cell(&Coordinates::new(row + 3, col))
                    {
                        game_state.winner = Some(player_index);
                        break;
                    }
                }
            }
        }

        // Check horizontal
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH - 3 {
                let cell = board.get_cell(&Coordinates::new(row, col));
                if let Cell::Stone(player_index) = cell {
                    if cell == board.get_cell(&Coordinates::new(row, col + 1))
                        && cell == board.get_cell(&Coordinates::new(row, col + 2))
                        && cell == board.get_cell(&Coordinates::new(row, col + 3))
                    {
                        game_state.winner = Some(player_index);
                        break;
                    }
                }
            }
        }

        // Check ascending diagonal
        for row in 3..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH - 3 {
                let cell = board.get_cell(&Coordinates::new(row, col));
                if let Cell::Stone(player_index) = cell {
                    if cell == board.get_cell(&Coordinates::new(row - 1, col + 1))
                        && cell == board.get_cell(&Coordinates::new(row - 2, col + 2))
                        && cell == board.get_cell(&Coordinates::new(row - 3, col + 3))
                    {
                        game_state.winner = Some(player_index);
                        break;
                    }
                }
            }
        }

        // Check diagonal descending
        for row in 0..BOARD_HEIGHT - 3 {
            for col in 0..BOARD_WIDTH - 3 {
                let cell = board.get_cell(&Coordinates::new(row, col));
                if let Cell::Stone(player_index) = cell {
                    if cell == board.get_cell(&Coordinates::new(row + 1, col + 1))
                        && cell == board.get_cell(&Coordinates::new(row + 2, col + 2))
                        && cell == board.get_cell(&Coordinates::new(row + 3, col + 3))
                    {
                        game_state.winner = Some(player_index);
                        break;
                    }
                }
            }
        }

        game_state
    }
}
