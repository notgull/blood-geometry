// Copyright 2023 John Nunley
//
// This file is part of blood-geometry.
// 
// blood-geometry is free software: you can redistribute it and/or modify it 
// under the terms of the GNU Affero General Public License as published by 
// the Free Software Foundation, either version 3 of the License, or (at your 
// option) any later version.
// 
// blood-geometry is distributed in the hope that it will be useful, but 
// WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY 
// or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License 
// for more details.
// 
// You should have received a copy of the GNU Affero General Public License 
// along with blood-geometry. If not, see <https://www.gnu.org/licenses/>. 

//! Composite operations.

/// An operation for compositing two surfaces together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CompositeOperation {
    /// Clear the destination surface.
    Clear,
}