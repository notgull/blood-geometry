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

//! A pure-Rust implementation of the Bentley-Ottmann algorithm.

#![cfg(feature = "alloc")]

use crate::line::LineSegment;
use crate::point::Point;
use crate::trapezoid::Trapezoid;
use crate::{ApproxEq, FillRule};

use core::{iter::FusedIterator, num::NonZeroUsize};
use num_traits::real::Real;

mod algorithm;
mod compare;

/// The whole point.
///
/// This function iterates over the intersections between the given
/// line segments. It returns an iterator over the intersections.
///
/// The iterator does not yield intersections lazily; the entire
/// `segments` iterator is consumed before the iterator is created.
pub(crate) fn bentley_ottmann<T: Real + ApproxEq>(
    segments: impl IntoIterator<Item = LineSegment<T>>,
) -> impl FusedIterator<Item = Point<T>> {
    bentley_ottmann_events(segments).filter_map(|event| {
        if matches!(event.event_type, EventType::Intersection { .. }) {
            Some(event.point)
        } else {
            None
        }
    })
}

/// Get an iterator over the Bentley-Ottmann algorithm's output.
///
/// This function returns an iterator over the Bentley-Ottmann algorithm's
/// events. The iterator yields all of the events, not just intersections.
///
/// The iterator does not yield intersections lazily; the entire
/// `segments` iterator is consumed before the iterator is created.
pub(crate) fn bentley_ottmann_events<T: Real + ApproxEq>(
    segments: impl IntoIterator<Item = LineSegment<T>>,
) -> BentleyOttmann<T> {
    BentleyOttmann {
        inner: algorithm::Algorithm::new(segments.into_iter(), ()),
    }
}

/// Rasterizes the polygon defined by the edges into trapezoids.
pub(crate) fn trapezoids<T: Real + ApproxEq>(
    segments: impl IntoIterator<Item = LineSegment<T>>,
    fill_rule: FillRule,
) -> Trapezoids<T> {
    Trapezoids {
        inner: algorithm::Algorithm::new(segments.into_iter(), fill_rule),
    }
}

/// An event that may occur in the Bentley-Ottmann algorithm.
#[derive(Debug, Clone)]
pub(crate) struct Event<Num: Copy> {
    /// The edge that this event is associated with.
    pub edge: LineSegment<Num>,

    /// The event type.
    pub event_type: EventType<Num>,

    /// The point that this event is associated with.
    pub point: Point<Num>,

    /// The index of the edge that this event is associated with.
    edge_id: NonZeroUsize,
}

/// The type of event that may occur in the Bentley-Ottmann algorithm.
#[derive(Debug, Clone)]
pub enum EventType<Num: Copy> {
    /// A start event, or the beginning of a segment.
    Start,

    /// A stop event, or the end of a segment.
    Stop,

    /// An intersection event.
    Intersection {
        /// The other edge we intersect with.
        other_edge: LineSegment<Num>,
    },
}

pub(crate) struct BentleyOttmann<Num: Copy> {
    inner: algorithm::Algorithm<Num, algorithm::NoTrapezoids>,
}

impl<Num: Real + ApproxEq> Iterator for BentleyOttmann<Num> {
    type Item = Event<Num>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_event()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let heap_len = self.inner.queue_len();

        (heap_len, Some(heap_len * 3))
    }
}

impl<Num: Real + ApproxEq> FusedIterator for BentleyOttmann<Num> {}

/// The return type of `Shape::trapezoids()`.
pub struct Trapezoids<Num: Copy> {
    inner: algorithm::Algorithm<Num, algorithm::Trapezoids<Num>>,
}

impl<Num: Real + ApproxEq> Iterator for Trapezoids<Num> {
    type Item = Trapezoid<Num>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next_trapezoid()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let traps = self.inner.trapezoid_len();
        (
            traps,
            Some(traps.saturating_add(self.inner.queue_len().saturating_mul(2))),
        )
    }
}

impl<Num: Real + ApproxEq> FusedIterator for Trapezoids<Num> {}
