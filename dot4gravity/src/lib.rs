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

#![cfg_attr(not(feature = "std"), no_std)]

use crate::traits::Bound;
use core::marker::PhantomData;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::{prelude::vec::Vec, TypeInfo};

#[cfg(test)]
mod tests;
mod traits;

const INITIAL_SEED: Seed = 123_456;
const INCREMENT: Seed = 74;
const MULTIPLIER: Seed = 75;
const MODULUS: Seed = Seed::pow(2, 16);

const BOARD_WIDTH: u8 = 10;
const BOARD_HEIGHT: u8 = 10;
const NUM_OF_PLAYERS: usize = 2;
const NUM_OF_BOMBS_PER_PLAYER: u8 = 3;
const NUM_OF_BLOCKS: u8 = 10;

// Score
const NB_POINT_STONE: u8 = 1;
const NB_POINT_ENEMY_STONE_DESTROYED: u8 = 1;

type PlayerIndex = u8;
type Position = u8;
type Seed = u32;
type Score = u8;

/// Represents a cell of the board.
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
        matches!(self, Cell::Empty | Cell::Bomb(_))
    }

    /// Tells if a cell must be cleared when it's affected by an explosion.
    fn is_explodable(&self) -> bool {
        *self != Cell::Block
    }

    /// Tells if a cell is suitable for dropping a stone.
    fn is_stone_droppable(&self) -> bool {
        !matches!(self, Cell::Block | Cell::Stone(_))
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

    fn random(seed: Seed) -> (Self, Seed) {
        let linear_congruential_generator = |seed: Seed| -> Seed {
            MULTIPLIER.saturating_mul(seed).saturating_add(INCREMENT) % MODULUS
        };

        let random_seed_1 = linear_congruential_generator(seed);
        let random_seed_2 = linear_congruential_generator(random_seed_1);

        (
            Coordinates::new(
                (random_seed_1 % (BOARD_WIDTH as Seed - 1)) as u8,
                (random_seed_2 % (BOARD_HEIGHT as Seed - 1)) as u8,
            ),
            random_seed_2,
        )
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
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    North,
    East,
    South,
    West,
}

impl Side {
    fn bound_coordinates(&self, position: Position) -> Coordinates {
        match self {
            Side::North => Coordinates::new(0, position),
            Side::South => Coordinates::new(BOARD_HEIGHT - 1, position),
            Side::West => Coordinates::new(position, 0),
            Side::East => Coordinates::new(position, BOARD_WIDTH - 1),
        }
    }
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

    fn is_stone_droppable(&self, position: &Coordinates) -> bool {
        position.is_inside_board() && self.get_cell(position).is_stone_droppable()
    }

    /// If the given cell position is a stone, return owner player index
    fn player_index_stone(&self, position: &Coordinates) -> Option<PlayerIndex> {
        if !position.is_inside_board() {
            return None;
        }
        if let Cell::Stone(p) = self.get_cell(position) {
            Some(p)
        } else {
            None
        }
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

    /// Return coordinates affected by a potential explosion
    fn explodable_coordinate(&self, position: &Coordinates) -> Vec<Coordinates> {
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
                    (row_offset + position.row as i8) as u8,
                    (col_offset + position.col as i8) as u8,
                )
            })
            .collect()
    }

    fn explode_bomb(&mut self, bomb_position: Coordinates) {
        self.explodable_coordinate(&bomb_position)
            .into_iter()
            .for_each(|position| {
                if self.is_explodable(&position) {
                    self.update_cell(position, Cell::Empty)
                }
            })
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
    /// Tried to drop a bomb outside bomb phase.
    DroppedBombOutsideBombPhase,
    /// Tried to drop a stone outside play phase.
    DroppedStoneOutsidePlayPhase,
    /// The player has no more bombs to drop.
    NoMoreBombsAvailable,
    /// Tried to drop a bomb in an invalid cell. The cell is already taken.
    InvalidBombPosition,
    /// Tried to drop a stone in an invalid cell. The cell is already taken.
    InvalidStonePosition,
    /// Tried to drop a stone during other player's turn
    NotPlayerTurn,
    /// The cell has no previous position. It is an edge cell.
    NoPreviousPosition,
    /// Tried playing when game has finished.
    GameAlreadyFinished,
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Copy, Clone, Debug, Eq, PartialEq)]
pub struct LastMove<Player> {
    pub player: Player,
    pub side: Side,
    pub position: Position,
}

impl<Player> LastMove<Player> {
    fn new(player: Player, side: Side, position: Position) -> Self {
        Self {
            player,
            side,
            position,
        }
    }
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Copy, Clone, Debug, Eq, PartialEq)]
pub struct GameState<Player> {
    /// Represents random seed.
    pub seed: Seed,
    /// Represents the game board.
    pub board: Board,
    /// Game mode.
    pub phase: GamePhase,
    /// When present,it contains the player that won.
    pub winner: Option<Player>,
    /// Next player turn.
    pub next_player: Player,
    /// Players:
    pub players: [Player; NUM_OF_PLAYERS],
    /// Number of bombs available for each player.
    pub bombs: [(Player, u8); NUM_OF_PLAYERS],
    /// Current score for each player.
    pub scores: [(Player, Score); NUM_OF_PLAYERS],
    /// Represents the last move.
    pub last_move: Option<LastMove<Player>>,
}

impl<Player: PartialEq + Clone> GameState<Player> {
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

    /// Return current player score
    pub fn get_player_score(&self, player: &Player) -> Score {
        self.scores
            .iter()
            .find(|(p, _)| *p == *player)
            .map(|(_, current_score)| *current_score)
            .unwrap()
    }

    /// Increase player score
    pub fn increase_player_score(&mut self, player: &Player, add_score: u8) {
        for (p, current_score) in self.scores.iter_mut() {
            if *p == *player {
                *current_score += add_score;
            }
        }
    }

    /// Return nb opponent player stones in the explodable area
    fn adjacent_opponent_stone(&self, position: Coordinates, player: &Player) -> u8 {
        let mut nb_adjacent_opponent_stone = 0;

        self.board
            .explodable_coordinate(&position)
            .into_iter()
            .for_each(|position| match self.board.player_index_stone(&position) {
                Some(player_index) if player_index != self.player_index(player) => {
                    nb_adjacent_opponent_stone += 1;
                }
                _ => {}
            });

        nb_adjacent_opponent_stone
    }

    pub fn is_player_turn(&self, player: &Player) -> bool {
        self.next_player == *player
    }
    fn player_index(&self, player: &Player) -> PlayerIndex {
        let player_index = self
            .players
            .iter()
            .position(|this_player| this_player == player)
            .expect("game to always start with 2 players") as u8;
        player_index
    }

    fn next_player(&self) -> &Player {
        let current_player_index = self
            .players
            .iter()
            .position(|player| *player == self.next_player)
            .expect("next player to be a subset of players");
        &self.players[(current_player_index + 1) % NUM_OF_PLAYERS]
    }
}

#[derive(Encode, Decode, TypeInfo)]
pub struct Game<Player>(PhantomData<Player>);

impl<Player: PartialEq + Clone> Game<Player> {
    fn can_drop_bomb(
        game_state: &GameState<Player>,
        position: &Coordinates,
        player: &Player,
    ) -> Result<(), GameError> {
        if game_state.phase != GamePhase::Bomb {
            return Err(GameError::DroppedBombOutsideBombPhase);
        }
        if game_state.winner.is_some() {
            return Err(GameError::GameAlreadyFinished);
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
        side: &Side,
        position: Position,
        player: &Player,
    ) -> Result<(), GameError> {
        if game_state.phase != GamePhase::Play {
            return Err(GameError::DroppedStoneOutsidePlayPhase);
        }
        if game_state.winner.is_some() {
            return Err(GameError::GameAlreadyFinished);
        }
        if !game_state.is_player_turn(player) {
            return Err(GameError::NotPlayerTurn);
        }
        if !game_state
            .board
            .is_stone_droppable(&side.bound_coordinates(position))
        {
            return Err(GameError::InvalidStonePosition);
        }
        Ok(())
    }
}

impl<Player: PartialEq + Clone> Game<Player> {
    /// Create a new game.
    pub fn new_game(player1: Player, player2: Player, seed: Option<Seed>) -> GameState<Player> {
        let mut board = Board::new();
        let mut blocks = Vec::new();
        let mut remaining_blocks = NUM_OF_BLOCKS;

        let mut seed = seed.unwrap_or(INITIAL_SEED);

        while remaining_blocks > 0 {
            let (block_coordinates, new_seed) = Coordinates::random(seed);
            seed = new_seed;
            if !blocks.contains(&block_coordinates) {
                blocks.push(block_coordinates);
                board.update_cell(block_coordinates, Cell::Block);
                remaining_blocks -= 1;
            }
        }

        GameState {
            seed,
            board,
            phase: Default::default(),
            winner: Default::default(),
            next_player: player1.clone(),
            players: [player1.clone(), player2.clone()],
            scores: [
                (player1.clone(), Score::default()),
                (player2.clone(), Score::default()),
            ],
            bombs: [
                (player1, NUM_OF_BOMBS_PER_PLAYER),
                (player2, NUM_OF_BOMBS_PER_PLAYER),
            ],
            last_move: Default::default(),
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
        position: Position,
    ) -> Result<GameState<Player>, GameError> {
        Self::can_drop_stone(&game_state, &side, position, &player)?;
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
                            game_state.increase_player_score(
                                &player,
                                NB_POINT_ENEMY_STONE_DESTROYED
                                    * game_state.adjacent_opponent_stone(position, &player),
                            );
                            game_state.board.explode_bomb(position);
                            stop = true;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side) {
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
                                    Coordinates::new(position.row.saturating_sub(1), position.col),
                                    Cell::Stone(player_index),
                                );
                            } else {
                                return Err(GameError::InvalidStonePosition);
                            }
                            stop = true;
                        }
                        // The stone is placed in the previous position of a stone.
                        Cell::Stone(_) => {
                            if row > 0 {
                                game_state.board.update_cell(
                                    Coordinates::new(position.row.saturating_sub(1), position.col),
                                    Cell::Stone(player_index),
                                );
                            } else {
                                return Err(GameError::InvalidStonePosition);
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
                            game_state.increase_player_score(
                                &player,
                                NB_POINT_ENEMY_STONE_DESTROYED
                                    * game_state.adjacent_opponent_stone(position, &player),
                            );
                            game_state.board.explode_bomb(position);
                            break;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side) {
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
                                return Err(GameError::InvalidStonePosition);
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
                                return Err(GameError::InvalidStonePosition);
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
                            game_state.increase_player_score(
                                &player,
                                NB_POINT_ENEMY_STONE_DESTROYED
                                    * game_state.adjacent_opponent_stone(position, &player),
                            );
                            game_state.board.explode_bomb(position);
                            break;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side) {
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
                                return Err(GameError::InvalidStonePosition);
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
                                return Err(GameError::InvalidStonePosition);
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
                            game_state.increase_player_score(
                                &player,
                                NB_POINT_ENEMY_STONE_DESTROYED
                                    * game_state.adjacent_opponent_stone(position, &player),
                            );
                            game_state.board.explode_bomb(position);
                            stop = true;
                        }
                        // The stone is placed at the end if it's empty.
                        Cell::Empty => {
                            if position.is_opposite_cell(side) {
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
                                    Coordinates::new(position.row, position.col.saturating_sub(1)),
                                    Cell::Stone(player_index),
                                );
                            } else {
                                return Err(GameError::InvalidStonePosition);
                            }
                            stop = true;
                        }
                        // The stone is placed in the previous position of a stone.
                        Cell::Stone(_) => {
                            if col < BOARD_WIDTH - 1 {
                                game_state.board.update_cell(
                                    Coordinates::new(position.row, position.col.saturating_sub(1)),
                                    Cell::Stone(player_index),
                                );
                            } else {
                                return Err(GameError::InvalidStonePosition);
                            }
                            stop = true;
                        }
                    }
                    col += 1;
                }
            }
        }

        game_state.increase_player_score(&player, NB_POINT_STONE);
        game_state.last_move = Some(LastMove::new(player, side, position));
        game_state.next_player = game_state.next_player().clone();
        game_state = Game::check_winner_player(game_state);

        Ok(game_state)
    }

    fn check_winner_player(mut game_state: GameState<Player>) -> GameState<Player> {
        if game_state.winner.is_some() {
            return game_state;
        }

        let board = &game_state.board;
        let mut squares = [0; NUM_OF_PLAYERS];

        for row in 0..BOARD_HEIGHT - 1 {
            for col in 0..BOARD_WIDTH - 1 {
                let cell = board.get_cell(&Coordinates::new(row, col));
                if let Cell::Stone(player_index) = cell {
                    if cell == board.get_cell(&Coordinates::new(row, col + 1))
                        && cell == board.get_cell(&Coordinates::new(row + 1, col))
                        && cell == board.get_cell(&Coordinates::new(row + 1, col + 1))
                    {
                        squares[player_index as usize] += 1;
                        if squares[player_index as usize] >= 3 {
                            let winner = game_state.players[player_index as usize].clone();
                            game_state.winner = Some(winner);
                            break;
                        }
                    }
                }
            }
        }

        game_state
    }
}
