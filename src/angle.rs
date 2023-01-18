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

use num_traits::Zero;
use num_traits::{float::FloatConst, real::Real};

use core::fmt;
use core::ops;

/// An angle between two points.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Angle<T>(T);

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Angle<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Angle(arbitrary::Arbitrary::arbitrary(u)?))
    }
}

impl<T: fmt::Debug> fmt::Debug for Angle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct DegreeWrapper<'a, T: fmt::Debug>(&'a T);

        impl<'a, T: fmt::Debug> fmt::Debug for DegreeWrapper<'a, T> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:?} rad", self.0)
            }
        }

        f.debug_tuple("Angle")
            .field(&DegreeWrapper(&self.0))
            .finish()
    }
}

impl<T> Angle<T> {
    /// Create a new `Angle` from the number of radians.
    pub fn from_radians(radians: T) -> Self {
        Angle(radians)
    }

    /// Create a new `Angle` from the number of degrees.
    pub fn from_degrees(degrees: T) -> Self
    where
        T: Real,
    {
        Angle(degrees.to_radians())
    }

    /// Get the number of radians in the angle.
    pub fn radians(self) -> T {
        self.0
    }

    /// Get the number of degrees in the angle.
    pub fn degrees(self) -> T
    where
        T: Real,
    {
        self.0.to_degrees()
    }

    /// Get the sine of the angle.
    pub fn sin(self) -> T
    where
        T: Real,
    {
        self.0.sin()
    }

    /// Get the cosine of the angle.
    pub fn cos(self) -> T
    where
        T: Real,
    {
        self.0.cos()
    }
}

impl<T> Angle<T>
where
    T: ops::Rem<Output = T>
        + ops::Sub<Output = T>
        + ops::Add<Output = T>
        + Zero
        + FloatConst
        + PartialOrd
        + Copy,
{
    /// Get the angle in the `[0..2*pi]` range.
    pub fn positive(self) -> Self {
        let two_pi = T::PI() + T::PI();
        let mut angle = self.0 % two_pi;

        // If the angle is negative, add 2*pi to it.
        if angle < T::zero() {
            angle = angle + two_pi;
        }

        Angle::from_radians(angle)
    }

    /// Get the angle in the `[-pi..pi]` range.
    pub fn signed(self) -> Self {
        todo!()
    }
}

impl<T: ops::Add<Output = T>> ops::Add for Angle<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Angle(self.0 + rhs.0)
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Angle<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Angle(self.0 - rhs.0)
    }
}

impl<T: ops::AddAssign> ops::AddAssign for Angle<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<T: ops::SubAssign> ops::SubAssign for Angle<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl<T: ops::Div<Output = T>> ops::Div for Angle<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Angle(self.0 / rhs.0)
    }
}

impl<T: ops::Mul<Output = T>> ops::Mul for Angle<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Angle(self.0 * rhs.0)
    }
}

impl<T: ops::DivAssign> ops::DivAssign for Angle<T> {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl<T: ops::MulAssign> ops::MulAssign for Angle<T> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl<T: ops::Div<Output = T> + Copy> ops::Div<T> for Angle<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Angle(self.0 / rhs)
    }
}

impl<T: ops::Mul<Output = T> + Copy> ops::Mul<T> for Angle<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Angle(self.0 * rhs)
    }
}

impl<T: ops::DivAssign + Copy> ops::DivAssign<T> for Angle<T> {
    fn div_assign(&mut self, rhs: T) {
        self.0 /= rhs;
    }
}

impl<T: ops::MulAssign + Copy> ops::MulAssign<T> for Angle<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs;
    }
}

impl<T: ops::Neg<Output = T>> ops::Neg for Angle<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Angle(-self.0)
    }
}
