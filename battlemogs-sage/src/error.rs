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

use sage_api::TransitionError;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TransitionErrorCode(u8);

impl From<TransitionErrorCode> for TransitionError {
	fn from(value: TransitionErrorCode) -> Self {
		TransitionError::Transition { code: value.0 }
	}
}
