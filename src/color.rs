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

//! Four-channel color type.

use core::fmt;
use core::ops;

use crate::pair::Quad;
use num_traits::{real::Real, AsPrimitive, Bounded};

/// Four-channel color type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Color<T: Copy>(Quad<T>);

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Color<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let components: [T; 4] = arbitrary::Arbitrary::arbitrary(u)?;
        Ok(Color::from_array(components))
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
#[serde(rename = "Color")]
struct LogicalColor<T> {
    red: T,
    green: T,
    blue: T,
    alpha: T,
}

#[cfg(feature = "serde")]
impl<T: Copy + serde::Serialize> serde::Serialize for Color<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        LogicalColor {
            red: self.0[0],
            green: self.0[1],
            blue: self.0[2],
            alpha: self.0[3],
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Copy + serde::Deserialize<'de>> serde::Deserialize<'de> for Color<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let LogicalColor {
            red,
            green,
            blue,
            alpha,
        } = serde::Deserialize::deserialize(deserializer)?;
        Ok(Color::new(red, green, blue, alpha))
    }
}

impl<T: fmt::Debug + Copy> fmt::Display for Color<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Color")
            .field("red", &self.0[0])
            .field("green", &self.0[1])
            .field("blue", &self.0[2])
            .field("alpha", &self.0[3])
            .finish()
    }
}

impl<T: Copy> Color<T> {
    /// Create a new `Color` from the red, green, blue and alpha components.
    pub fn new(red: T, green: T, blue: T, alpha: T) -> Self {
        Color(Quad::new([red, green, blue, alpha]))
    }

    /// Create a new `Color` from an array of red, green, blue and alpha components.
    pub fn from_array(array: [T; 4]) -> Self {
        Color(Quad::new(array))
    }

    /// Convert the `Color` into an array of red, green, blue and alpha components.
    pub fn into_array(self) -> [T; 4] {
        self.0.into_inner()
    }

    /// Get the red component of the `Color`.
    pub fn red(&self) -> T {
        self.0[0]
    }

    /// Get the green component of the `Color`.
    pub fn green(&self) -> T {
        self.0[1]
    }

    /// Get the blue component of the `Color`.
    pub fn blue(&self) -> T {
        self.0[2]
    }

    /// Get the alpha component of the `Color`.
    pub fn alpha(&self) -> T {
        self.0[3]
    }
}

impl<T: Copy> Color<T> {
    /// Premultiply the `Color` to other components.
    ///
    /// This is useful for converting the `Color` to another format.
    pub fn multiply<U: ops::Add<Output = U> + Bounded + AsPrimitive<T>>(self) -> Color<U>
    where
        T: Real + AsPrimitive<U>,
    {
        macro_rules! cvt {
            ($e:expr) => {{
                ($e * U::max_value().as_()).as_() + U::min_value()
            }};
        }

        Color::new(
            cvt!(self.red()),
            cvt!(self.green()),
            cvt!(self.blue()),
            cvt!(self.alpha()),
        )
    }

    /// Divide this color into floating point components.
    pub fn divide<U: Real + AsPrimitive<T>>(self) -> Color<U>
    where
        T: ops::Sub<Output = T> + Bounded + AsPrimitive<U>,
    {
        macro_rules! cvt {
            ($e:expr) => {{
                ($e - T::min_value()).as_() / T::max_value().as_()
            }};
        }

        Color::new(
            cvt!(self.red()),
            cvt!(self.green()),
            cvt!(self.blue()),
            cvt!(self.alpha()),
        )
    }
}
