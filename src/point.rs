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

use crate::pair::{Double, Quad};
use crate::transform::Transformable;
use crate::ApproxEq;

use core::cmp;
use core::fmt;
use core::hash::{self, Hash};
use core::ops;

use num_traits::real::Real;
use num_traits::{One, Signed, Zero};

macro_rules! two_dimensional {
    (
        $(#[$outer:meta])*
        $name:ident ($mint_name: ident, $euclid_name:ident, $kurbo_name:ident)
        $diff:ident
    ) => {
        $(#[$outer])*
        #[derive(Copy, Clone)]
        //#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable, bytemuck::Pod))]
        #[repr(transparent)]
        pub struct $name<T: Copy>(pub(crate) Double<T>);

        impl<T: Copy + fmt::Debug> fmt::Debug for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.x())
                    .field(&self.y())
                    .finish()
            }
        }

        impl<T: Copy + PartialEq> PartialEq for $name<T> {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl<T: Copy + Eq> Eq for $name<T> {}

        impl<T: Copy + PartialOrd> PartialOrd for $name<T> {
            fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }

        impl<T: Copy + Ord> Ord for $name<T> {
            fn cmp(&self, other: &Self) -> cmp::Ordering {
                self.0.cmp(&other.0)
            }
        }

        impl<T: Copy + Hash> Hash for $name<T> {
            fn hash<H: hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl<T: Copy + Default> Default for $name<T> {
            fn default() -> Self {
                Self(Double::default())
            }
        }

        #[cfg(feature = "arbitrary")]
        impl<'a, T: arbitrary::Arbitrary<'a> + Copy> arbitrary::Arbitrary<'a> for $name<T> {
            fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
                let (x, y) = arbitrary::Arbitrary::arbitrary(u)?;
                Ok(Self::new(x, y))
            }
        }

        #[cfg(feature = "serde")]
        impl<T: Copy + serde::Serialize> serde::Serialize for $name<T> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                (&self.x(), &self.y()).serialize(serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de, T: Copy + serde::Deserialize<'de>> serde::Deserialize<'de> for $name<T> {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let (x, y) = serde::Deserialize::deserialize(deserializer)?;
                Ok(Self(Double::new([x, y])))
            }
        }

        impl<T: Copy> $name<T> {
            /// Get the X coordinate.
            #[inline]
            pub fn x(self) -> T {
                self.0[0]
            }

            /// Get the Y coordinate.
            #[inline]
            pub fn y(self) -> T {
                self.0[1]
            }

            /// Constructor of two elements.
            #[inline]
            pub fn new(x: T, y: T) -> Self {
                $name(Double::new([x, y]))
            }

            /// Constructor with the same X and Y coordinates.
            #[inline]
            pub fn splat(value: T) -> Self {
                $name(Double::splat(value))
            }

            /// Constructor with an array of its coordinates.
            #[inline]
            pub fn from_array(array: [T; 2]) -> Self {
                $name(Double::new(array))
            }

            /// Constructor with a tuple of its coordinates.
            #[inline]
            pub fn from_tuple((a, b): (T, T)) -> Self {
                $name(Double::new([a, b]))
            }
        }

        impl<T: Copy + Zero> $name<T> {
            /// Constructor with zero coordinates.
            #[inline]
            pub fn zero() -> Self {
                $name(Double::splat(T::zero()))
            }
        }

        impl<T: Copy> From<[T; 2]> for $name<T> {
            #[inline]
            fn from(array: [T; 2]) -> Self {
                $name::from_array(array)
            }
        }

        impl<T: Copy> From<(T, T)> for $name<T> {
            #[inline]
            fn from(tuple: (T, T)) -> Self {
                $name::from_tuple(tuple)
            }
        }

        impl<T: Copy> From<$name<T>> for [T; 2] {
            #[inline]
            fn from(point: $name<T>) -> Self {
                point.0.into_inner()
            }
        }

        impl<T: Copy> From<$name<T>> for (T, T) {
            #[inline]
            fn from(point: $name<T>) -> Self {
                let [a, b] = point.0.into_inner();
                (a, b)
            }
        }

        #[cfg(feature = "mint")]
        impl<T: Copy> From<mint::$mint_name<T>> for $name<T> {
            #[inline]
            fn from(point: mint::$mint_name<T>) -> Self {
                let array: [T; 2] = point.into();
                array.into()
            }
        }

        #[cfg(feature = "mint")]
        impl<T: Copy> From<$name<T>> for mint::$mint_name<T> {
            #[inline]
            fn from(point: $name<T>) -> Self {
                let [x, y] = point.0.into_inner();
                mint::$mint_name { x, y }
            }
        }

        #[cfg(feature = "euclid")]
        impl<T: Copy, U> From<euclid::$euclid_name<T, U>> for $name<T> {
            #[inline]
            fn from(point: euclid::$euclid_name<T, U>) -> Self {
                let array: [T; 2] = point.into();
                array.into()
            }
        }

        #[cfg(feature = "euclid")]
        impl<T: Copy, U> From<$name<T>> for euclid::$euclid_name<T, U> {
            #[inline]
            fn from(point: $name<T>) -> Self {
                let [x, y] = point.0.into_inner();
                euclid::$euclid_name::new(x, y)
            }
        }

        #[cfg(feature = "kurbo")]
        impl From<kurbo::$kurbo_name> for $name<f64> {
            #[inline]
            fn from(point: kurbo::$kurbo_name) -> Self {
                let kurbo::$kurbo_name { x, y } = point;
                [x, y].into()
            }
        }

        #[cfg(feature = "kurbo")]
        impl From<$name<f64>> for kurbo::$kurbo_name {
            #[inline]
            fn from(point: $name<f64>) -> Self {
                let [x, y] = point.0.into_inner();
                kurbo::$kurbo_name { x, y }
            }
        }

        impl<T: Copy + ops::Add<Output = T>> ops::Add<$diff<T>> for $name<T> {
            type Output = Self;

            #[inline]
            fn add(self, other: $diff<T>) -> Self {
                $name(self.0 + other.0)
            }
        }

        impl<T: Copy + ops::AddAssign> ops::AddAssign<$diff<T>> for $name<T> {
            #[inline]
            fn add_assign(&mut self, other: $diff<T>) {
                self.0 += other.0;
            }
        }

        impl<T: Copy + ops::Sub<Output = T>> ops::Sub<$diff<T>> for $name<T> {
            type Output = Self;

            #[inline]
            fn sub(self, other: $diff<T>) -> Self {
                $name(self.0 - other.0)
            }
        }

        impl<T: Copy + ops::SubAssign> ops::SubAssign<$diff<T>> for $name<T> {
            #[inline]
            fn sub_assign(&mut self, other: $diff<T>) {
                self.0 -= other.0;
            }
        }

        impl<T: Copy + ops::Mul<Output = T>> ops::Mul<T> for $name<T> {
            type Output = Self;

            #[inline]
            fn mul(self, other: T) -> Self {
                $name(self.0 * Double::splat(other))
            }
        }

        impl<T: Copy + ops::MulAssign> ops::MulAssign<T> for $name<T> {
            #[inline]
            fn mul_assign(&mut self, other: T) {
                self.0 *= Double::splat(other);
            }
        }

        impl<T: Copy + ops::Mul<Output = T>> ops::Mul<$diff<T>> for $name<T> {
            type Output = Self;

            #[inline]
            fn mul(self, other: $diff<T>) -> Self {
                $name(self.0 * other.0)
            }
        }

        impl<T: Copy + ops::MulAssign> ops::MulAssign<$diff<T>> for $name<T> {
            #[inline]
            fn mul_assign(&mut self, other: $diff<T>) {
                self.0 *= other.0;
            }
        }

        impl<T: Copy + ops::Div<Output = T>> ops::Div<T> for $name<T> {
            type Output = Self;

            #[inline]
            fn div(self, other: T) -> Self {
                $name(self.0 / Double::splat(other))
            }
        }

        impl<T: Copy + ops::Div<Output = T>> ops::Div<$diff<T>> for $name<T> {
            type Output = Self;

            #[inline]
            fn div(self, other: $diff<T>) -> Self {
                $name(self.0 / other.0)
            }
        }

        impl<T: Copy + ops::DivAssign> ops::DivAssign<T> for $name<T> {
            #[inline]
            fn div_assign(&mut self, other: T) {
                self.0 /= Double::splat(other);
            }
        }

        impl<T: Copy + ops::DivAssign> ops::DivAssign<$diff<T>> for $name<T> {
            #[inline]
            fn div_assign(&mut self, other: $diff<T>) {
                self.0 /= other.0;
            }
        }

        impl<T: Copy + ops::Neg<Output = T>> ops::Neg for $name<T> {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self {
                $name(-self.0)
            }
        }

        impl<T: Copy> $name<T> {
            /// Get the absolute value of all coordinates.
            #[inline]
            pub fn abs(self) -> Self where T: Signed {
                $name(self.0.abs())
            }

            /// Get the minimum value of all coordinates.
            #[inline]
            pub fn min(self, other: Self) -> Self where T: PartialOrd {
                $name(self.0.min(other.0))
            }

            /// Get the maximum value of all coordinates.
            #[inline]
            pub fn max(self, other: Self) -> Self where T: PartialOrd {
                $name(self.0.max(other.0))
            }

            /// Clamp the coordinates to the range `[min, max]`.
            #[inline]
            pub fn clamp(self, min: Self, max: Self) -> Self where T: PartialOrd {
                $name(self.0.clamp(min.0, max.0))
            }

            /// Linearly interpolate between two sets of coordinates.
            #[inline]
            pub fn lerp(self, other: Self, t: T) -> Self where
                T: One + ops::Sub<Output = T> + ops::Mul<Output = T> + ops::Add<Output = T> {
                let one_t = T::one() - t;

                // Combine them into a quad for a multiplication, then add them together.
                let points = Quad::from_double(self.0, other.0);
                let multiplier = Quad::new([one_t, one_t, t, t]);
                let result = points * multiplier;

                let (point1, point2) = result.split();
                $name(point1 + point2)
            }

            /// Round the coordinates to the nearest integer.
            #[inline]
            pub fn round(self) -> Self where T: Real {
                $name(self.0.round())
            }

            /// Round the coordinates down.
            #[inline]
            pub fn floor(self) -> Self where T: Real {
                $name(self.0.floor())
            }

            /// Round the coordinates up.
            #[inline]
            pub fn ceil(self) -> Self where T: Real {
                $name(self.0.ceil())
            }
        }

        impl<T: Copy + ApproxEq> $name<T> {
            /// Check if all coordinates are approximately equal to another point.
            #[inline]
            pub fn approx_eq(&self, other: &Self) -> bool {
                self.x().approx_eq(&other.x()) &&
                self.y().approx_eq(&other.y())
            }
        }
    }
}

two_dimensional! {
    /// A two-dimensional point in space.
    Point (Point2, Point2D, Point)
    Vector
}

two_dimensional! {
    /// A two-dimensional vector describing the distance between two points.
    Vector (Vector2, Vector2D, Vec2)
    Vector
}

impl<T: Copy + ops::Sub<Output = T>> ops::Sub<Point<T>> for Point<T> {
    type Output = Vector<T>;

    #[inline]
    fn sub(self, other: Point<T>) -> Vector<T> {
        Vector(self.0 - other.0)
    }
}

impl<T: Copy> From<Vector<T>> for Point<T> {
    #[inline]
    fn from(vector: Vector<T>) -> Self {
        Point(vector.0)
    }
}

impl<T: Copy> From<Point<T>> for Vector<T> {
    #[inline]
    fn from(point: Point<T>) -> Self {
        Vector(point.0)
    }
}

impl<T: Copy> Point<T> {
    /// Convert this point to a vector.
    pub fn into_vector(self) -> Vector<T> {
        Vector(self.0)
    }
}

impl<T: Copy> Vector<T> {
    /// Convert this vector to a point.
    pub fn into_point(self) -> Point<T> {
        Point(self.0)
    }

    /// Get the length of the vector.
    #[inline]
    pub fn length(self) -> T
    where
        T: Real,
    {
        self.length_squared().sqrt()
    }

    /// Get the dot product of two vectors.
    #[inline]
    pub fn dot(self, other: Self) -> T
    where
        T: ops::Add<Output = T> + ops::Mul<Output = T>,
    {
        // Get x and y product.
        let products = self.0 * other.0;

        // Add them together.
        let [x, y] = products.into_inner();
        x + y
    }

    /// Get the cross product of two vectors.
    #[inline]
    pub fn cross(self, other: Self) -> T
    where
        T: ops::Sub<Output = T> + ops::Mul<Output = T>,
    {
        let other = other.0.swap();
        let products = self.0 * other;
        let [x1y2, x2y1] = products.into_inner();
        x1y2 - x2y1
    }

    /// Get the square length of this vector.
    #[inline]
    pub fn length_squared(self) -> T
    where
        T: ops::Add<Output = T> + ops::Mul<Output = T>,
    {
        self.dot(self)
    }

    /// Normalize this vector so that it has a length of one.
    #[inline]
    pub fn normalize(self) -> Self
    where
        T: Real,
    {
        self / self.length()
    }

    /// Project this vector onto another vector.
    #[inline]
    pub fn project(self, other: Self) -> Self
    where
        T: Real,
    {
        other.scale_uniform(self.dot(other) / other.length_squared())
    }
}

impl<T: Copy> Point<T> {
    /// Get the distance between this point and another point.
    #[inline]
    pub fn distance(self, other: Self) -> T
    where
        T: Real,
    {
        (self - other).length()
    }

    /// Get the distance squared between this point and another point.
    #[inline]
    pub fn distance_squared(self, other: Self) -> T
    where
        T: ops::Add<Output = T> + ops::Sub<Output = T> + ops::Mul<Output = T>,
    {
        (self - other).length_squared()
    }

    /// Get the midpoint between this point and another point.
    #[inline]
    pub fn midpoint(self, other: Self) -> Self
    where
        T: ops::Add<Output = T> + ops::Div<Output = T> + One,
    {
        let sum = self.0 + other.0;
        Self(sum / Double::splat(T::one() + T::one()))
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, Vector};

    #[test]
    fn test_point() {
        let point = Point::new(1.0, 2.0);
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
    }

    #[test]
    fn test_vector() {
        let vector = Vector::new(1.0, 2.0);
        assert_eq!(vector.x(), 1.0);
        assert_eq!(vector.y(), 2.0);
    }

    #[test]
    fn test_add() {
        let point = Point::new(1.0, 2.0);
        let vector = Vector::new(3.0, 4.0);
        let result = point + vector;
        assert_eq!(result.x(), 4.0);
        assert_eq!(result.y(), 6.0);
    }

    #[test]
    fn test_sub() {
        let point = Point::new(1.0, 2.0);
        let vector = Vector::new(3.0, 4.0);
        let result = point - vector;
        assert_eq!(result.x(), -2.0);
        assert_eq!(result.y(), -2.0);
    }

    #[test]
    fn test_mul() {
        let point = Point::new(1.0, 2.0);
        let result = point * 2.0;
        assert_eq!(result.x(), 2.0);
        assert_eq!(result.y(), 4.0);
    }

    #[test]
    fn test_div() {
        let point = Point::new(1.0, 2.0);
        let result = point / 2.0;
        assert_eq!(result.x(), 0.5);
        assert_eq!(result.y(), 1.0);
    }

    #[test]
    fn test_neg() {
        let point = Point::new(1.0, 2.0);
        let result = -point;
        assert_eq!(result.x(), -1.0);
        assert_eq!(result.y(), -2.0);
    }

    #[test]
    fn test_dot() {
        let vector1 = Vector::new(1.0, 2.0);
        let vector2 = Vector::new(3.0, 4.0);
        let result = vector1.dot(vector2);
        assert_eq!(result, 11.0);
    }

    #[test]
    fn test_cross() {
        let vector1 = Vector::new(1.0, 2.0);
        let vector2 = Vector::new(3.0, 4.0);
        let result = vector1.cross(vector2);
        assert_eq!(result, -2.0);
    }

    #[test]
    fn test_length() {
        let vector = Vector::new(3.0, 4.0);
        let result = vector.length();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_length_squared() {
        let vector = Vector::new(3.0, 4.0);
        let result = vector.length_squared();
        assert_eq!(result, 25.0);
    }

    #[test]
    fn test_normalize() {
        let vector = Vector::new(3.0, 4.0);
        let result = vector.normalize();
        assert_eq!(result.x(), 0.6);
        assert_eq!(result.y(), 0.8);
    }

    #[test]
    fn test_project() {
        let vector1 = Vector::new(3.0, 4.0);
        let vector2 = Vector::new(1.0, 2.0);
        let result = vector1.project(vector2);
        assert_eq!(result.x(), 1.2);
        assert_eq!(result.y(), 2.4);
    }

    #[test]
    fn test_distance() {
        let point1 = Point::new(1.0, 2.0);
        let point2 = Point::new(1.0, 4.0);
        let result = point1.distance(point2);
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_distance_squared() {
        let point1 = Point::new(1.0, 2.0);
        let point2 = Point::new(1.0, 4.0);
        let result = point1.distance_squared(point2);
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_midpoint() {
        let point1 = Point::new(1.0, 2.0);
        let point2 = Point::new(1.0, 4.0);
        let result = point1.midpoint(point2);
        assert_eq!(result.x(), 1.0);
        assert_eq!(result.y(), 3.0);
    }

    #[test]
    fn test_from() {
        let point = Point::new(1.0, 2.0);
        let vector = Vector::from(point);
        assert_eq!(vector.x(), 1.0);
        assert_eq!(vector.y(), 2.0);
    }

    #[test]
    fn test_into() {
        let vector = Vector::new(1.0, 2.0);
        let point: Point<f32> = vector.into();
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
    }

    #[test]
    fn test_from_array() {
        let array = [1.0, 2.0];
        let point = Point::from(array);
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
    }

    #[test]
    fn test_into_array() {
        let point = Point::new(1.0, 2.0);
        let array: [f32; 2] = point.into();
        assert_eq!(array[0], 1.0);
        assert_eq!(array[1], 2.0);
    }

    #[test]
    fn test_from_tuple() {
        let tuple = (1.0, 2.0);
        let point = Point::from(tuple);
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
    }
}
