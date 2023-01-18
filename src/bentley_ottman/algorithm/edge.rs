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

use crate::bentley_ottman::{Event, EventType};
use crate::line::{Line, NhLineSegment};
use crate::point::Point;
use crate::trapezoid::Trapezoid;
use crate::ApproxEq;
use alloc::{boxed::Box, vec::Vec};
use core::{
    cell::{Cell, RefCell},
    fmt,
    num::NonZeroUsize,
};
use num_traits::real::Real;

/// An edge to be used in the algorithm.
#[derive(Debug)]
pub(crate) struct BoEdge<Num: Copy> {
    /// The segment that this edge represents.
    edge: NhLineSegment<Num>,

    /// The point in this edge with the lowest Y value.
    lowest_y: Point<Num>,

    /// The point in this edge with the highest Y value.
    highest_y: Point<Num>,

    /// An ID number that uniquely refers to this edge.
    ///
    /// This is the index number of this edge in the edges
    /// array, plus one, so that we can take advantage of
    /// niching optimizations in the algorithm.
    id: NonZeroUsize,

    /// The previous edge in the sweep line.
    prev: Cell<Option<NonZeroUsize>>,

    /// The next edge in the sweep line.
    next: Cell<Option<NonZeroUsize>>,

    /// The partial trapezoid that this edge is building up to.
    ///
    /// This edge is considered to be the left edge of the
    /// trapezoid. The trapezoid itself contains the top
    /// coordinate and the right edge.
    trapezoid: RefCell<Option<PartialTrapezoid<Num>>>,
}

/// A trapezoid that has not been entirely completed yet.
#[derive(Debug)]
struct PartialTrapezoid<Num> {
    /// The edge ID associated with the right edge of this trapezoid.
    right_edge: NonZeroUsize,

    /// The top coordinate of this trapezoid.
    top: Num,
}

/// A static list of all available edges.
///
/// It is invariant that the edges in this list cannot
/// be modified except for interior mutability.
pub(crate) struct Edges<Num: Copy> {
    edges: Box<[BoEdge<Num>]>,
}

impl<Num: Copy> BoEdge<Num> {
    /// Get the edge ID of this edge.
    pub(super) fn id(&self) -> NonZeroUsize {
        self.id
    }

    /// Get the segment associated with this edge.
    pub(super) fn edge(&self) -> NhLineSegment<Num> {
        self.edge
    }

    /// Get the point in this edge with the lowest Y value.
    pub(super) fn lowest_y(&self) -> Point<Num> {
        self.lowest_y
    }

    /// Get the point in this edge with the highest Y value.
    pub(super) fn highest_y(&self) -> Point<Num> {
        self.highest_y
    }

    /// Get the ID of the previous edge in the sweep line.
    pub(super) fn prev(&self) -> Option<NonZeroUsize> {
        self.prev.get()
    }

    /// Set the ID of the previous edge in the sweep line.
    pub(super) fn set_prev(&self, prev: Option<NonZeroUsize>) {
        self.prev.set(prev);
    }

    /// Get the ID of the next edge in the sweep line.
    pub(super) fn next(&self) -> Option<NonZeroUsize> {
        self.next.get()
    }

    /// Set the ID of the next edge in the sweep line.
    pub(super) fn set_next(&self, next: Option<NonZeroUsize>) {
        self.next.set(next);
    }

    /// Tell whether or not we have a pending trapezoid.
    pub(super) fn pending_trapezoid(&self) -> bool {
        self.trapezoid.borrow().is_some()
    }

    /// Swap the partial trapezoid to this edge from another.
    pub(super) fn take_trapezoid(&self, other: &Self) {
        *self.trapezoid.borrow_mut() = other.trapezoid.borrow_mut().take();
    }
}

impl<Num: Real + ApproxEq> BoEdge<Num> {
    /// Get the start event for this edge.
    pub(super) fn start_event(&self) -> Event<Num> {
        Event {
            edge: self.edge().into(),
            event_type: EventType::Start,
            point: self.lowest_y(),
            edge_id: self.id(),
        }
    }

    /// Get the end event for this edge.
    pub(super) fn stop_event(&self) -> Event<Num> {
        Event {
            edge: self.edge().into(),
            event_type: EventType::Stop,
            point: self.highest_y(),
            edge_id: self.id(),
        }
    }

    /// Complete the trapezoid for this edge at a given Y value.
    pub(super) fn complete_trapezoid(
        &self,
        bottom: Num,
        all: &Edges<Num>,
    ) -> Option<Trapezoid<Num>> {
        self.trapezoid
            .borrow_mut()
            .take()
            .and_then(|trap| trap.complete(self.id(), bottom, all))
    }

    /// Create a `BoEdge` from two points.
    ///
    /// Only used in testing.
    #[cfg(test)]
    pub(super) fn from_points(point1: Point<Num>, point2: Point<Num>, id: NonZeroUsize) -> Self
    where
        Num: fmt::Debug,
    {
        use crate::LineSegment;
        use core::convert::TryInto;
        let edge = LineSegment::new(point1, point2);
        Self::from_edge(edge.try_into().unwrap(), id)
    }

    /// Is this edge colinear with another edge?
    pub(super) fn colinear(&self, other: &BoEdge<Num>) -> bool {
        let line = self.edge.line();
        let point = other.edge.line().origin();
        let vector = other.edge.line().direction();

        // we know that point and point + vector are on the same line
        // if both points are a distance of zero from our line's equation,
        // we know that they are colinear
        line.distance(point).approx_eq(&Num::zero())
            && line.distance(point + vector).approx_eq(&Num::zero())
    }

    /// Get the X coordinate for this edge at a given Y coordinate.
    pub(super) fn x_at_y(&self, y: Num) -> Num {
        x_for_y(&self.edge.line(), y)
    }

    /// Either start a new trapezoid or continue an existing one.
    ///
    /// If a trapezoid was finished using this function, this returns
    /// that trapezoid.
    pub(super) fn start_trapezoid(
        &self,
        right: &BoEdge<Num>,
        top: Num,
        all: &Edges<Num>,
    ) -> Option<Trapezoid<Num>> {
        let mut trap = self.trapezoid.borrow_mut();

        // if the current trapezoid is not empty, we may need to either
        // extend it or end it
        let mut completed_trap = None;

        if let Some(ref mut inner_trap) = &mut *trap {
            // if the trap has the same right edge as the one we're about to
            // install, fail early
            if inner_trap.right_edge == right.id() {
                return None;
            }

            // if the other line is colinear to the current right edge,
            // just extend the trapezoid to there
            if all.get(inner_trap.right_edge).colinear(right) {
                inner_trap.right_edge = right.id();

                // we're done
                return None;
            } else {
                // otherwise, we need to end the trapezoid
                // hopefully the rust compiler can optimize out the unwrap
                completed_trap = trap.take().unwrap().complete(self.id(), top, all);
            }
        };

        // create a new trapezoid
        let trapezoid = PartialTrapezoid {
            right_edge: right.id(),
            top,
        };
        *trap = Some(trapezoid);

        completed_trap
    }

    /// Get the intersection event between this edge and another edge.
    ///
    /// This does not preform the check to see if the intersection may
    /// have already occurred.
    pub(super) fn intersection_event(&self, other: &BoEdge<Num>) -> Option<Event<Num>> {
        self.edge.intersection(&other.edge).map(|point| Event {
            edge: self.edge().into(),
            event_type: EventType::Intersection {
                other_edge: other.edge().into(),
            },
            point,
            edge_id: self.id(),
        })
    }

    /// Create a new `BoEdge` from an `Edge` and its ID number.
    pub(super) fn from_edge(edge: NhLineSegment<Num>, id: NonZeroUsize) -> Self {
        // get the points of the edge
        debug_assert!(edge.top() <= edge.bottom());
        let lowest_y = Point::new(x_for_y(&edge.line(), edge.top()), edge.top());
        let highest_y = Point::new(x_for_y(&edge.line(), edge.bottom()), edge.bottom());

        Self {
            edge,
            lowest_y,
            highest_y,
            id,
            prev: Cell::new(None),
            next: Cell::new(None),
            trapezoid: RefCell::new(None),
        }
    }
}

impl<Num: Copy + PartialOrd> PartialTrapezoid<Num> {
    /// Complete this trapezoid.
    fn complete(
        self,
        left_edge: NonZeroUsize,
        bottom: Num,
        all: &Edges<Num>,
    ) -> Option<Trapezoid<Num>> {
        let Self { right_edge, top } = self;
        let left = all.get(left_edge);
        let right = all.get(right_edge);

        // if the bottom comes before the top, this trapezoid
        // is invalid
        if bottom < top {
            None
        } else {
            Some(Trapezoid::new(
                top,
                bottom,
                left.edge().line(),
                right.edge().line(),
            ))
        }
    }
}

impl<Num: Copy> Edges<Num> {
    /// Get the edge at the given index.
    pub(super) fn get(&self, index: NonZeroUsize) -> &BoEdge<Num> {
        self.edges
            .get(index.get() - 1)
            .expect("index out of bounds")
    }
}

impl<Num: Copy> From<Vec<BoEdge<Num>>> for Edges<Num> {
    fn from(edges: Vec<BoEdge<Num>>) -> Self {
        Edges {
            edges: edges.into_boxed_slice(),
        }
    }
}

impl<Num: Copy> IntoIterator for Edges<Num> {
    type Item = BoEdge<Num>;
    type IntoIter = alloc::vec::IntoIter<BoEdge<Num>>;

    fn into_iter(self) -> Self::IntoIter {
        self.edges.into_vec().into_iter()
    }
}

impl<'a, Num: Copy> IntoIterator for &'a Edges<Num> {
    type Item = &'a BoEdge<Num>;
    type IntoIter = core::slice::Iter<'a, BoEdge<Num>>;

    fn into_iter(self) -> Self::IntoIter {
        self.edges.iter()
    }
}

impl<Num: fmt::Debug + Copy> fmt::Debug for Edges<Num> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Edges").field(&self.edges).finish()
    }
}

/// Calculate the X for a given Y value.
///
/// # Panics
///
/// This function will panic if the provided line is horizontal.
/// However, the algorithm filters out horizontal lines automatically,
/// so this should never happen.
fn x_for_y<Num: Real + ApproxEq>(line: &Line<Num>, y: Num) -> Num {
    line.point_at_y(y).expect("horizontal line").x()
}
