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

//! Quadratic Bezier curves.

use crate::path::{Path, PathEvent};
use crate::{point::Point, ApproxEq, Curve, LineSegment};
use num_traits::{real::Real, One};

use core::ops;

/// A quadratic Bezier curve.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct QuadraticBezier<T: Copy>([Point<T>; 3]);

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for QuadraticBezier<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let pts: [Point<T>; 3] = u.arbitrary()?;
        Ok(QuadraticBezier(pts))
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
#[serde(rename = "QuadraticBezier")]
struct LogicalQuad<T: Copy> {
    from: Point<T>,
    control: Point<T>,
    to: Point<T>,
}

#[cfg(feature = "serde")]
impl<T: Copy + serde::Serialize> serde::Serialize for QuadraticBezier<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        LogicalQuad {
            from: self.0[0],
            control: self.0[1],
            to: self.0[2],
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Copy + serde::Deserialize<'de>> serde::Deserialize<'de> for QuadraticBezier<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let LogicalQuad { from, control, to } = serde::Deserialize::deserialize(deserializer)?;
        Ok(QuadraticBezier([from, control, to]))
    }
}

impl<T: Copy> QuadraticBezier<T> {
    /// Create a new quadratic Bezier curve.
    pub fn new(p1: Point<T>, p2: Point<T>, p3: Point<T>) -> Self {
        QuadraticBezier([p1, p2, p3])
    }

    /// Get the three points of the quadratic Bezier curve.
    pub fn points(&self) -> [Point<T>; 3] {
        self.0
    }

    /// Get the origin point of the quadratic Bezier curve.
    #[inline]
    pub fn from(&self) -> Point<T> {
        self.0[0]
    }

    /// Get the control point of the quadratic Bezier curve.
    #[inline]
    pub fn control(&self) -> Point<T> {
        self.0[1]
    }

    /// Get the destination point of the quadratic Bezier curve.
    #[inline]
    pub fn to(&self) -> Point<T> {
        self.0[2]
    }

    /// Can this curve be approximated with a single line segment?
    pub fn is_linear(&self, tolerance: T) -> bool
    where
        T: PartialOrd
            + One
            + ops::Add<Output = T>
            + ops::Sub<Output = T>
            + ops::Mul<Output = T>
            + ops::Div<Output = T>,
    {
        if self.from() == self.to() {
            return true;
        }

        let line = LineSegment::new(self.to(), self.from());
        let dist = line.line().distance_squared(self.control());

        let four = T::one() + T::one() + T::one() + T::one();
        dist <= tolerance * tolerance * four
    }

    /// Get the uncurved version of this curve.
    pub fn baseline(&self) -> LineSegment<T> {
        LineSegment::new(self.from(), self.to())
    }
}

impl<T: Real + ApproxEq> Curve<T> for QuadraticBezier<T> {
    type FlattenIterator = FlattenedQuad<T>;
    type Subsection = Self;
    type Derivative = LineSegment<T>;

    fn eval(&self, t: T) -> Point<T> {
        let mt = T::one() - t;
        let mt2 = mt * mt;
        let t2 = t * t;
        let two = T::one() + T::one();

        let p1 = self.0[0] * mt2;
        let p2 = self.0[1] * two * mt * t;
        let p3 = self.0[2] * t2;

        Point(p1.0 + p2.0 + p3.0)
    }

    fn flatten(&self, tolerance: T) -> Self::FlattenIterator {
        FlattenedQuad::new(*self, tolerance)
    }

    fn split(self, index: T) -> (Self::Subsection, Self::Subsection) {
        let mid = self.eval(index);
        let left = QuadraticBezier::new(self.from(), self.from().lerp(self.control(), index), mid);
        let right = QuadraticBezier::new(mid, self.control().lerp(self.to(), index), self.to());
        (left, right)
    }

    fn subsection(self, range: ops::Range<T>) -> Self::Subsection {
        let (t0, t1) = (range.start, range.end);

        let from = self.eval(t0);
        let to = self.eval(t1);
        let ctrl =
            from + (self.control() - self.from()).lerp(self.to() - self.control(), t0) * (t1 - t0);

        Self([from, ctrl, to])
    }

    // Taken from https://docs.rs/kurbo/latest/src/kurbo/quadbez.rs.html#239-279
    fn length(&self, _accuracy: T) -> T {
        macro_rules! t {
            ($e:expr) => {
                T::from($e).unwrap()
            };
        }

        let [p0, p1, p2] = self.0;

        let two = T::one() + T::one();
        let half = T::one() / two;
        let quarter = half / two;

        let d2 = p0.into_vector() - (p1.into_vector() * two) + p2.into_vector();
        let a = d2.length_squared();
        let d1 = p1 - p0;
        let c = d1.length_squared();

        let nearly_straight = T::from(5e-4).unwrap();
        if a < nearly_straight * c {
            // This case happens for nearly straight Béziers.
            //
            // Calculate arclength using Legendre-Gauss quadrature using formula from Behdad
            // in https://github.com/Pomax/BezierInfo-2/issues/77
            let v0 = ((p0.into_vector() * t!(-0.492943519233745))
                + (p1.into_vector() * t!(0.430331482911935))
                + (p2.into_vector() * t!(0.0626120363218102)))
            .length();
            let v1 = ((p2 - p0) * t!(0.4444444444444444)).length();
            let v2 = ((p0.into_vector() * t!(-0.0626120363218102))
                - (p1.into_vector() * t!(0.430331482911935))
                + (p2.into_vector() * t!(0.492943519233745)))
            .length();
            return v0 + v1 + v2;
        }
        let b = d2.dot(d1) * two;

        let sabc = (a + b + c).sqrt();
        let a2 = a.powf(-half);
        let a32 = a2.powi(3);
        let c2 = two * c.sqrt();
        let ba_c2 = b * a2 + c2;

        let v0 = quarter * a2 * a2 * b * (two * sabc - c2) + sabc;
        // TODO: justify and fine-tune this exact constant.
        let tiny = t!(1e-12);
        if ba_c2 < tiny {
            // This case happens for Béziers with a sharp kink.
            v0
        } else {
            v0 + quarter
                * a32
                * (two * two * c * a - b * b)
                * (((two * a + b) * a2 + two * sabc) / ba_c2).ln()
        }
    }

    fn derivative(&self) -> Self::Derivative {
        let [p1, p2, p3] = self.0;
        let two = T::one() + T::one();

        LineSegment::new((p2 - p1).into_point() * two, (p3 - p2).into_point() * two)
    }
}

impl<T: Copy> Path<T> for QuadraticBezier<T> {
    type Iter = crate::iter::Three<PathEvent<T>>;

    fn path_iter(self) -> Self::Iter {
        crate::iter::Three::from([
            PathEvent::Begin { at: self.from() },
            PathEvent::Quadratic {
                from: self.from(),
                control: self.control(),
                to: self.to(),
            },
            PathEvent::End {
                last: self.to(),
                first: self.from(),
                close: false,
            },
        ])
    }

    fn rectilinear(self) -> bool {
        false
    }

    fn approximate_length(self, accuracy: T) -> T
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        self.length(accuracy)
    }
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct FlattenedQuad<T: Copy> {
    curve: QuadraticBezier<T>,
    out: bool,
    inner: FlattenedInner<T>,
}

// Implements this algorithm: https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html
impl<T: Real> FlattenedQuad<T> {
    fn new(curve: QuadraticBezier<T>, tolerance: T) -> Self {
        Self {
            inner: FlattenedInner::new(&curve, tolerance),
            curve,
            out: false,
        }
    }
}

impl<T: Real + ApproxEq> Iterator for FlattenedQuad<T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.out {
            None
        } else {
            match self.inner.next() {
                Some(t) => Some(self.curve.eval(t)),
                None => {
                    self.out = true;
                    Some(self.curve.to())
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Approximates the values of (1 + 4x^2)^-0.25 dx, used in the flattening process.
fn approx_parabola_integral<T: Real>(value: T) -> T {
    let two_thirds = (T::one() + T::one()) / (T::one() + T::one() + T::one());
    let one_quarter = T::one() / (T::one() + T::one() + T::one() + T::one());

    value
        / (T::one() - two_thirds
            + (two_thirds.powi(4) + one_quarter * value * value)
                .sqrt()
                .sqrt())
}

/// Approximates the inverse of the function demonstrated above.
fn approx_parabola_inv_integral<T: Real>(value: T) -> T {
    let b: T = T::from::<f32>(0.39).expect("can convert f32 to T");
    let one_quarter = T::one() / (T::one() + T::one() + T::one() + T::one());

    value * (T::one() - b + (b * b + one_quarter * value * value).sqrt())
}

#[derive(Debug, Clone)]
pub(crate) struct FlattenedInner<T: Copy> {
    index: T,
    integral_from: T,
    integral_step: T,
    inv_integral_from: T,
    div_inv_integral_diff: T,
    count: T,
}

impl<T: Real> FlattenedInner<T> {
    pub(crate) fn new(curve: &QuadraticBezier<T>, tolerance: T) -> Self {
        // Map the curve to a parabola.
        let [from, control, to] = curve.points();
        let two = T::one() + T::one();
        let half = T::one() / two;

        let change = (control * two) - from - to.into_vector();
        let distance = to - from;
        let cross = distance.cross(change);

        let inv_cross = cross.recip();
        let parabola_from = (control - from) * change;
        let parabola_to = (to - control) * change;
        let parabola_from = parabola_from.x() + (parabola_from.y() * inv_cross);
        let parabola_to = parabola_to.x() + (parabola_to.y() * inv_cross);

        let scale = cross.abs() / (change.length() * (parabola_from - parabola_to).abs());

        let integral_from = approx_parabola_integral(parabola_from);
        let integral_to = approx_parabola_integral(parabola_to);
        let integral_diff = integral_to - integral_from;

        let inv_integral_from = approx_parabola_inv_integral(integral_from);
        let inv_integral_to = approx_parabola_inv_integral(integral_to);
        let div_inv_integral_diff = T::one() / (inv_integral_to - inv_integral_from);

        // TODO(notgull): Check divisors to prevent NaNs.
        let count = (half * integral_diff.abs() * (scale / tolerance).sqrt()).ceil();

        let integral_step = integral_diff / count;

        Self {
            index: T::zero(),
            integral_from,
            integral_step,
            inv_integral_from,
            div_inv_integral_diff,
            count,
        }
    }

    fn t_for_iteration(&self, iteration: T) -> T {
        let u = approx_parabola_inv_integral(self.integral_from + self.integral_step * iteration);
        (u - self.inv_integral_from) * self.div_inv_integral_diff
    }
}

impl<T: Real> Iterator for FlattenedInner<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            None
        } else {
            let t = self.t_for_iteration(self.index);
            self.index = self.index + T::one();
            Some(t)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.count + T::one() - self.index).to_usize().unwrap_or(0);
        (size, Some(size))
    }
}
