// Copyright 2022 Capsule Corp (France) SAS.
// This file is part of Ternoa.

// Ternoa is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Ternoa is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Ternoa.  If not, see <http://www.gnu.org/licenses/>.

pub fn check_bounds<T>(src_len: usize, min_len: (u32, T), max_len: (u32, T)) -> Result<(), T> {
	if src_len < min_len.0 as usize {
		return Err(min_len.1)
	}
	if src_len > max_len.0 as usize {
		return Err(max_len.1)
	}

	Ok(())
}
