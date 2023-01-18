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

//! Iterator over the line segments of a path.

use crate::path::flatten::Flattened;
use crate::path::{PathEvent, StraightPathEvent};
use crate::{ApproxEq, LineSegment};

use num_traits::real::Real;

/// The iterator returned by `Path::segments`.
#[derive(Debug, Clone)]
pub struct LineSegments<T: Copy, P>(pub(crate) Flattened<T, P>);

impl<T: Real + ApproxEq, P: Iterator<Item = PathEvent<T>>> Iterator for LineSegments<T, P> {
    type Item = LineSegment<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some(StraightPathEvent::Begin { .. })
                | Some(StraightPathEvent::End { close: false, .. }) => continue,
                Some(StraightPathEvent::Line { from, to })
                | Some(StraightPathEvent::End {
                    first: to,
                    last: from,
                    close: true,
                }) => return Some(LineSegment::new(from, to)),
                _ => unreachable!(),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
