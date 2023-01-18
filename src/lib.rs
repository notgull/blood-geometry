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

//! Various geometric primitives.
//! 
//! `blood-geometry` is a toolkit that provides a wide variety of 
//! two-dimensional geometric primitives. The goal is for this project to be 
//! a "one-stop shop" for geometry-related functionality.
//! 
//! Out of the box, `blood-geometry` provides the following:
//! 
//! * Point, vector, rectangle and size types.
//! * Quadratic and cubic bezier curves.
//! * A variety of shape types.
//! * Traits for dealing with paths and shapes.
//! * Matrix trnasforms.
//! 
//! With the `alloc` feature enabled, `blood-geometry` also provides ways to
//! rasterize shapes and paths.

#![no_std]
#![forbid(unsafe_code, future_incompatible)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod angle;
mod arc;
mod bentley_ottman;
mod box2d;
mod color;
pub mod curve;
mod iter;
mod line;
mod pair;
pub mod path;
mod point;
mod rect;
pub mod region;
mod size;
mod transform;
mod trapezoid;
mod triangle;

pub use angle::Angle;
pub use arc::Arc;
pub use box2d::{BoundingBox, Box};
pub use color::Color;
pub use curve::{CubicBezier, Curve, QuadraticBezier};
pub use iter::{Four, Three, Two};
pub use line::{Line, LineSegment, NhLineSegment};
pub use path::{Path, PathBuffer, PathEvent, Shape, StraightPathEvent, Verb};
pub use point::{Point, Vector};
pub use rect::Rect;
pub use size::Size;
pub use transform::{Affine, Rotation, Scale, Transform, Translation};
pub use trapezoid::Trapezoid;
pub use triangle::Triangle;

use core::num::Wrapping;

/// A general-purpose "direction" type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// The direction is forwards.
    Forwards,

    /// The direction is backwards.
    Backwards,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Forwards
    }
}

/// Simple trait for telling if one value is approximately equal to another.
pub trait ApproxEq {
    /// Returns true if the values are approximately equal.
    fn approx_eq(&self, other: &Self) -> bool;
}

macro_rules! approx_eq_int_impl {
    ($($t:ty),*) => {
        $(
            impl ApproxEq for $t {
                #[inline]
                fn approx_eq(&self, other: &Self) -> bool {
                    self == other
                }
            }
        )*
    };
}

approx_eq_int_impl! {
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize
}

impl ApproxEq for f32 {
    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        (self - other).abs() < f32::EPSILON
    }
}

impl ApproxEq for f64 {
    #[inline]
    fn approx_eq(&self, other: &Self) -> bool {
        (self - other).abs() < f64::EPSILON
    }
}

impl<T: ApproxEq> ApproxEq for &T {
    fn approx_eq(&self, other: &Self) -> bool {
        T::approx_eq(*self, *other)
    }
}

impl<T: ApproxEq> ApproxEq for Wrapping<T> {
    fn approx_eq(&self, other: &Self) -> bool {
        T::approx_eq(&self.0, &other.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FillRule {
    Winding,
    EvenOdd,
}
