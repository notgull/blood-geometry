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

//! Flatten a path to straight lines.

use super::{Path, PathEvent, StraightPathEvent};
use num_traits::real::Real;

use crate::point::Point;
use crate::{ApproxEq, CubicBezier, Curve, QuadraticBezier};

#[derive(Debug, Clone)]
pub struct Flattened<T: Copy, P> {
    /// The path iterator we're flattening.
    iter: P,

    /// The tolerance for flattening.
    tolerance: T,

    /// The current state of the iterator.
    state: State<T>,
}

#[derive(Debug, Clone)]
enum State<T: Copy> {
    /// The iterator is not flattening any curves.
    None,

    /// The iterator is flattening a quadratic bezier curve.
    Quadratic {
        /// The inner iterator.
        iter: crate::curve::quad::FlattenedQuad<T>,

        /// The last point to form a line segment with.
        last: Option<Point<T>>,
    },

    /// The iterator is flattening a cubic bezier curve.
    Cubic {
        /// The inner iterator.
        iter: crate::curve::cubic::FlattenedCubic<T>,

        /// The last point to form a line segment with.
        last: Option<Point<T>>,
    },
}

impl<T: Real + ApproxEq, P> Flattened<T, P> {
    pub(crate) fn new(iter: P, tolerance: T) -> Self {
        Self {
            iter,
            tolerance,
            state: State::None,
        }
    }
}

impl<T: Real + ApproxEq, P: Iterator<Item = PathEvent<T>>> Iterator for Flattened<T, P> {
    type Item = StraightPathEvent<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we are flattening a curve, we need to check if we are done with it.
            if let Some(next) = self.state.next() {
                return Some(next);
            } else {
                self.state = State::None;
            }

            // If we are not flattening a curve, we need to check if we need to start flattening one.
            match self.iter.next() {
                None => return None,
                Some(PathEvent::Begin { at }) => return Some(StraightPathEvent::Begin { at }),
                Some(PathEvent::End { first, last, close }) => {
                    return Some(StraightPathEvent::End { first, last, close })
                }
                Some(PathEvent::Line { from, to }) => {
                    return Some(StraightPathEvent::Line { from, to })
                }
                Some(PathEvent::Quadratic { from, control, to }) => {
                    self.state = State::Quadratic {
                        iter: Curve::flatten(
                            &QuadraticBezier::new(from, control, to),
                            self.tolerance,
                        ),
                        last: None,
                    };

                    continue;
                }
                Some(PathEvent::Cubic {
                    from,
                    control1,
                    control2,
                    to,
                }) => {
                    self.state = State::Cubic {
                        iter: CubicBezier::new(from, control1, control2, to)
                            .flatten(self.tolerance),
                        last: None,
                    };

                    continue;
                }
                _ => unreachable!(),
            }
        }
    }
}

impl<T: Real + ApproxEq> State<T> {
    /// Yield the next event from the iterator.
    fn next(&mut self) -> Option<StraightPathEvent<T>> {
        match self {
            Self::None => None,
            Self::Cubic { iter, last } => loop {
                let current_point = iter.next()?;
                let last_point = match last.replace(current_point) {
                    Some(last_point) => last_point,
                    None => continue,
                };

                return Some(StraightPathEvent::Line {
                    from: last_point,
                    to: current_point,
                });
            },
            Self::Quadratic { iter, last } => loop {
                let current_point = iter.next()?;
                let last_point = match last.replace(current_point) {
                    Some(last_point) => last_point,
                    None => continue,
                };

                return Some(StraightPathEvent::Line {
                    from: last_point,
                    to: current_point,
                });
            },
        }
    }
}

impl<T: Real + ApproxEq, P: Iterator<Item = PathEvent<T>>> Path<T> for Flattened<T, P> {
    type Iter = FlattenedPathIter<T, P>;

    fn path_iter(self) -> Self::Iter {
        FlattenedPathIter(self)
    }
}

#[doc(hidden)]
pub struct FlattenedPathIter<T: Copy, P>(Flattened<T, P>);

impl<T: Real + ApproxEq, P: Iterator<Item = PathEvent<T>>> Iterator for FlattenedPathIter<T, P> {
    type Item = PathEvent<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| e.into())
    }
}
