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

//! Cubic Bezier curve.

use num_traits::real::Real;

use super::quad::{FlattenedInner as FlattenedQuad, QuadraticBezier};
use crate::{point::Point, ApproxEq, Curve};

/// A cubic bezier curve.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CubicBezier<T: Copy>([Point<T>; 4]);

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for CubicBezier<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let pts: [Point<T>; 4] = u.arbitrary()?;
        Ok(CubicBezier(pts))
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
#[serde(rename = "CubicBezier")]
struct LogicalCubic<T: Copy> {
    from: Point<T>,
    control1: Point<T>,
    control2: Point<T>,
    to: Point<T>,
}

#[cfg(feature = "serde")]
impl<T: Copy + serde::Serialize> serde::Serialize for CubicBezier<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        LogicalCubic {
            from: self.0[0],
            control1: self.0[1],
            control2: self.0[2],
            to: self.0[3],
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Copy + serde::Deserialize<'de>> serde::Deserialize<'de> for CubicBezier<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let LogicalCubic {
            from,
            control1,
            control2,
            to,
        } = serde::Deserialize::deserialize(deserializer)?;
        Ok(CubicBezier([from, control1, control2, to]))
    }
}

impl<T: Copy> CubicBezier<T> {
    /// Create a new cubic Bezier curve.
    pub fn new(p1: Point<T>, p2: Point<T>, p3: Point<T>, p4: Point<T>) -> Self {
        CubicBezier([p1, p2, p3, p4])
    }

    /// Get the four points of the cubic Bezier curve.
    pub fn points(&self) -> [Point<T>; 4] {
        self.0
    }

    /// Get the origin point of the cubic Bezier curve.
    #[inline]
    pub fn from(&self) -> Point<T> {
        self.0[0]
    }

    /// Get the first control point of the cubic Bezier curve.
    #[inline]
    pub fn control1(&self) -> Point<T> {
        self.0[1]
    }

    /// Get the second control point of the cubic Bezier curve.
    #[inline]
    pub fn control2(&self) -> Point<T> {
        self.0[2]
    }

    /// Get the destination point of the cubic Bezier curve.
    #[inline]
    pub fn to(&self) -> Point<T> {
        self.0[3]
    }

    /// Convert this curve to its closest approximation as a quadratic
    /// Bezier curve.
    #[inline]
    pub fn as_quadratic(&self) -> QuadraticBezier<T>
    where
        T: Real,
    {
        let three = T::one() + T::one() + T::one();
        let half = T::one() / (T::one() + T::one());
        let control1 = (self.control1().into_vector() * three - self.from().into_vector()) * half;
        let control2 = (self.control2().into_vector() * three - self.to().into_vector()) * half;

        QuadraticBezier::new(
            self.from(),
            control1.into_point().midpoint(control2.into_point()),
            self.to(),
        )
    }

    fn gauss_arclen(&self, coeffs: &[(T, T)]) -> T
    where
        T: Real + ApproxEq,
    {
        let deriv = self.derivative();
        let half = T::one() / (T::one() + T::one());

        coeffs
            .iter()
            .map(|(wi, xi)| *wi * deriv.eval(half * (*xi + T::one())).into_vector().length())
            .fold(T::zero(), |a, b| a + b)
    }
}

impl<T: Real + ApproxEq> Curve<T> for CubicBezier<T> {
    type Subsection = Self;
    type FlattenIterator = FlattenedCubic<T>;
    type Derivative = QuadraticBezier<T>;

    fn eval(&self, t: T) -> Point<T> {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = T::one() - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        let three = T::one() + T::one() + T::one();

        let [from, control1, control2, to] = self.0;
        let p1 = from * mt3;
        let p2 = control1 * three * mt2 * t;
        let p3 = control2 * three * mt * t2;
        let p4 = to * t3;

        Point(p1.0 + p2.0 + p3.0 + p4.0)
    }

    fn flatten(&self, tolerance: T) -> Self::FlattenIterator {
        FlattenedCubic::new(self, tolerance)
    }

    // Taken from https://docs.rs/lyon_geom/latest/src/lyon_geom/cubic_bezier.rs.html#177
    fn split(self, index: T) -> (Self::Subsection, Self::Subsection) {
        let [from, control1, control2, to] = self.0;

        let p1 = from + (control1 - from) * index;
        let p2 = control1 + (control2 - control1) * index;
        let p3 = p1 + (p2 - p1) * index;
        let p4 = control2 + (to - control2) * index;
        let p5 = p2 + (p4 - p2) * index;
        let p6 = p3 + (p5 - p3) * index;

        (
            CubicBezier([from, p1, p3, p6]),
            CubicBezier([p6, p5, p4, to]),
        )
    }

    fn subsection(self, range: core::ops::Range<T>) -> Self::Subsection {
        let (t0, t1) = (range.start, range.end);
        let from = self.eval(t0);
        let to = self.eval(t1);

        let quad = QuadraticBezier::new(
            (self.control1() - self.from()).into_point(),
            (self.control2() - self.control1()).into_point(),
            (self.to() - self.control2()).into_point(),
        );

        let dt = t1 - t0;
        let ctrl1 = from + quad.eval(t0 * dt).into_vector() * dt;
        let ctrl2 = to - quad.eval(t1 * dt).into_vector() * dt;

        Self::new(from, ctrl1, ctrl2, to)
    }

    fn length(&self, accuracy: T) -> T {
        // Taken from https://docs.rs/kurbo/latest/src/kurbo/cubicbez.rs.html#431-472
        const MAX_DEPTH: usize = 16;

        fn cubic_errnorm<T: Real + ApproxEq>(c: &CubicBezier<T>) -> T {
            let deriv = c.derivative().derivative();
            let deriv2 = deriv.to() - deriv.from();
            let one_third = T::one() / (T::one() + T::one() + T::one());

            deriv.from().into_vector().length_squared()
                + deriv.from().into_vector().dot(deriv2)
                + deriv2.length_squared() * one_third
        }

        fn est_gauss9_error<T: Real + ApproxEq>(c: &CubicBezier<T>) -> T {
            let [p0, p1, p2, p3] = c.0;

            let lc2 = (p3 - p0).length_squared();
            let lp = (p1 - p0).length() + (p2 - p1).length() + (p3 - p2).length();
            let small_number = T::from(256e-8).unwrap();

            small_number * (cubic_errnorm(c) / lc2).powi(8) * lp
        }

        fn rec<T: Real + ApproxEq>(c: &CubicBezier<T>, accuracy: T, depth: usize) -> T {
            macro_rules! t {
                ($e:expr) => {
                    T::from($e).unwrap()
                };
            }

            // Magic coefficient numbers.
            let coeefs = [
                (t!(0.3302393550012598), t!(0.0000000000000000)),
                (t!(0.1806481606948574), t!(-0.8360311073266358)),
                (t!(0.1806481606948574), t!(0.8360311073266358)),
                (t!(0.0812743883615744), t!(-0.9681602395076261)),
                (t!(0.0812743883615744), t!(0.9681602395076261)),
                (t!(0.3123470770400029), t!(-0.3242534234038089)),
                (t!(0.3123470770400029), t!(0.3242534234038089)),
                (t!(0.2606106964029354), t!(-0.6133714327005904)),
                (t!(0.2606106964029354), t!(0.6133714327005904)),
            ];

            if depth == MAX_DEPTH || est_gauss9_error(c) < accuracy {
                c.gauss_arclen(&coeefs)
            } else {
                let one_half = T::one() / (T::one() + T::one());
                let (c0, c1) = c.split(one_half);
                rec(&c0, accuracy * one_half, depth + 1) + rec(&c1, accuracy * one_half, depth + 1)
            }
        }

        // Check if the bezier curve is degenerate, or almost degenerate
        // A degenerate curve where all points are identical will cause infinite recursion in the rec function (well, until MAX_DEPTH at least) in all branches.
        // This test will in addition be true if the bezier curve is just a simple line (i.e. p0=p1 and p2=p3).
        // The constant 0.5 has not been mathematically proven to be small enough, but from empirical tests
        // a value of about 0.87 should be enough. Thus 0.5 is a conservative value.
        // See https://github.com/linebender/kurbo/pull/100 for more info.
        let [p0, p1, p2, p3] = self.0;
        let one_half = T::one() / (T::one() + T::one());
        if (p1 - p0).length_squared() + (p2 - p3).length_squared() <= one_half * accuracy * accuracy
        {
            (p0 - p3).length()
        } else {
            rec(self, accuracy, 0)
        }
    }

    fn derivative(&self) -> Self::Derivative {
        let [p0, p1, p2, p3] = self.0;
        let three = T::one() + T::one() + T::one();

        QuadraticBezier::new(
            (p1 - p0).into_point() * three,
            (p2 - p1).into_point() * three,
            (p3 - p2).into_point() * three,
        )
    }
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct FlattenedCubic<T: Copy> {
    curve: CubicBezier<T>,
    current_quad: FlattenedQuad<T>,
    remaining: usize,
    tolerance: T,
    range_step: T,
    range_start: T,
}

impl<T: Real + ApproxEq> FlattenedCubic<T> {
    fn new(curve: &CubicBezier<T>, tolerance: T) -> Self {
        let quad_tolerance = tolerance * T::from(0.4).unwrap();
        let flat_tolerance = tolerance * T::from(0.8).unwrap();

        // Get the number of quadratics needed to approximate the curve.
        let [from, control1, control2, to] = curve.0;
        let three = T::one() + T::one() + T::one();
        let six = three + three;
        let err = from.into_vector() - (control1.into_vector() * three)
            + (control2.into_vector() * three)
            - to.into_vector();
        let err = err.length_squared();

        let num_quads = (err / (T::from(432.0).unwrap() * tolerance * tolerance))
            .powf(T::one() / six)
            .ceil()
            .max(T::one());

        let range_step = T::one() / num_quads;

        // Start flattening the initial quad segment.
        let current_quad = curve.subsection(T::zero()..range_step).as_quadratic();
        let current_quad = FlattenedQuad::new(&current_quad, quad_tolerance);

        FlattenedCubic {
            curve: *curve,
            current_quad,
            remaining: num_quads.to_usize().unwrap() - 1,
            tolerance: flat_tolerance,
            range_step,
            range_start: T::zero(),
        }
    }
}

impl<T: Real + ApproxEq> Iterator for FlattenedCubic<T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Try to get the next point from the current quad.
        if let Some(point) = self.current_quad.next() {
            return Some(self.curve.eval(point));
        }

        // If we're out of quads, we're done.
        if self.remaining == 0 {
            return None;
        }

        // Otherwise, start flattening the next quad segment.
        self.range_start = self.range_start + self.range_step;
        let t0 = self.range_start;
        let t1 = self.range_start + self.range_step;
        self.remaining -= 1;

        let quad = self.curve.subsection(t0..t1).as_quadratic();
        self.current_quad = FlattenedQuad::new(&quad, self.tolerance);

        let t_inner = self.current_quad.next().unwrap_or_else(T::one);
        let t = t0 + t_inner * self.range_step;

        Some(self.curve.eval(t))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining * self.current_quad.size_hint().0, None)
    }
}
