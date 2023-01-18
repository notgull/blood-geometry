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

//! Various types of curves.

use crate::point::Point;
use core::ops::Range;

pub(crate) mod cubic;
pub(crate) mod quad;

pub use cubic::CubicBezier;
pub use quad::QuadraticBezier;

/// Represents a curve that can be evaluated at a given parameter.
pub trait Curve<T: Copy>: Sized {
    /// An iterator that can be used to evaluate the flattened curve.
    type FlattenIterator: Iterator<Item = Point<T>>;

    /// The result of taking a subsection of the curve.
    type Subsection: Curve<T>;

    /// The type of the derivative of the curve.
    type Derivative;

    /// Evaluate the curve at the given parameter.
    fn eval(&self, t: T) -> Point<T>;

    /// Flatten the curve into a sequence of line segments.
    fn flatten(&self, tolerance: T) -> Self::FlattenIterator;

    /// Split this curve in half at the given point.
    fn split(self, index: T) -> (Self::Subsection, Self::Subsection);

    /// Split out a subsection of the curve defined by a range of indices.
    fn subsection(self, range: Range<T>) -> Self::Subsection;

    /// Get the total length of the curve.
    fn length(&self, accuracy: T) -> T;

    /// Get the derivative of the curve.
    fn derivative(&self) -> Self::Derivative;
}

impl<T: Copy, C: Curve<T> + Copy> Curve<T> for &C {
    type FlattenIterator = C::FlattenIterator;
    type Subsection = C::Subsection;
    type Derivative = C::Derivative;

    #[inline]
    fn eval(&self, t: T) -> Point<T> {
        (**self).eval(t)
    }

    #[inline]
    fn flatten(&self, tolerance: T) -> Self::FlattenIterator {
        Curve::flatten(&**self, tolerance)
    }

    #[inline]
    fn split(self, index: T) -> (Self::Subsection, Self::Subsection) {
        (*self).split(index)
    }

    #[inline]
    fn subsection(self, range: Range<T>) -> Self::Subsection {
        (*self).subsection(range)
    }

    #[inline]
    fn length(&self, accuracy: T) -> T {
        (**self).length(accuracy)
    }

    #[inline]
    fn derivative(&self) -> Self::Derivative {
        (**self).derivative()
    }
}
