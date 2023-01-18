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

//! Two-wide and four-wde vector types.
//! 
//! The intention is to replace these with SIMD types once they are stable.

#![allow(unused)]

macro_rules! eat_ident {
    ($i:ident) => {}
}

macro_rules! vector_type {
    (
        $(#[$attr:meta])*
        $name:ident([$($field:ident),+]) [$sz:expr] $modname:ident
    ) => {
        mod $modname {
            use core::ops::{self, Index, IndexMut};
            use num_traits::real::Real;
            
            macro_rules! implement_ops {
                ($n:ident,$oname:ident,$fname:ident) => {
                    impl<T: Copy +ops::$oname<Output = T>> ops::$oname for $n<T> {
                        type Output = $n<T>;

                        fn $fname(self, other: $n<T>) -> $n<T> {
                            let mut index = 0;

                            $(
                                let $field = self.0[index].$fname(other.0[index]);
                                index += 1;
                            )*

                            $n([$($field),*])
                        }
                    }
                };
                ($n:ident,$oname:ident,$fname:ident,$aoname:ident,$afname:ident) => {
                    implement_ops!($n,$oname,$fname);

                    impl<T: Copy + ops::$aoname> ops::$aoname for $n<T> {
                        fn $afname(&mut self, other: $n<T>) {
                            let mut index = 0;

                            $(
                                eat_ident!($field);
                                self.0[index].$afname(other.0[index]);
                                index += 1;
                            )*
                        }
                    }
                }
            }

            macro_rules! implement_sfold {
                (T: $trai: path, $n:ident,$oname:ident) => {
                    impl<T: $trai> $n<T> {
                        #[inline]
                        pub(crate) fn $oname(self) -> Self {
                            let Self([$($field),*]) = self;

                            $(
                                let $field = $field.$oname();
                            )*
                            
                            Self([$($field),*])
                        }
                    }
                }
            }

            macro_rules! implement_packed {
                (T: $trai: path,$n:ident,$oname:ident,$outtype:path,$clos:expr) => {
                    impl<T: Copy + $trai> $n<T> {
                        #[allow(clippy::redundant_closure_call)]
                        #[inline]
                        pub(crate) fn $oname(self, other: Self) -> $n<$outtype> {
                            let mut index = 0;

                            $(
                                let $field: $outtype = ($clos)(self.0[index], other.0[index]);
                                index += 1;
                            )*

                            $n([$($field),*])
                        }
                    }
                }
            }

            $(#[$attr])*
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub(crate) struct $name<T>(pub(super) [T; $sz]);

            impl<T> $name<T> {
                /// Create a new vector.
                pub(crate) fn new(array: [T; $sz]) -> Self {
                    $name(array)
                }

                /// Create a vector where every element is the same.
                pub(crate) fn splat(value: T) -> Self
                where
                    T: Copy,
                {
                    $name([value; $sz])
                }

                /// Unwrap into the inner array.
                pub(crate) fn into_inner(self) -> [T; $sz] {
                    self.0
                }

                /// Get the minimum value in the vector.
                pub(crate) fn min(self, other: Self) -> Self
                where
                    T: Copy + PartialOrd,
                {
                    let mut index = 0;

                    $(
                        let $field = if self.0[index] < other.0[index] {
                            self.0[index]
                        } else {
                            other.0[index]
                        };
                        index += 1;
                    )*

                    $name([$($field),*])
                } 

                /// Get the maximum value in the vector.
                pub(crate) fn max(self, other: Self) -> Self
                where
                    T: Copy + PartialOrd,
                {
                    let mut index = 0;

                    $(
                        let $field = if self.0[index] > other.0[index] {
                            self.0[index]
                        } else {
                            other.0[index]
                        };
                        index += 1;
                    )*

                    $name([$($field),*])
                }

                /// Clamp the vector between two other vectors.
                pub(crate) fn clamp(self, min: Self, max: Self) -> Self
                where
                    T: Copy + PartialOrd,
                {
                    let mut index = 0;

                    $(
                        let $field = if self.0[index] < min.0[index] {
                            min.0[index]
                        } else if self.0[index] > max.0[index] {
                            max.0[index]
                        } else {
                            self.0[index]
                        };
                        index += 1;
                    )*

                    $name([$($field),*])
                }
            }

            impl<T> Default for $name<T>
            where
                T: Default,
            {
                fn default() -> Self {
                    $name([$({
                        eat_ident!($field);
                        T::default()
                    }),*])
                }
            }

            impl<T> Index<usize> for $name<T> {
                type Output = T;

                fn index(&self, index: usize) -> &T {
                    &self.0[index]
                }
            }

            impl<T> IndexMut<usize> for $name<T> {
                fn index_mut(&mut self, index: usize) -> &mut T {
                    &mut self.0[index]
                }
            }

            implement_ops! {
                $name, Add, add, AddAssign, add_assign
            }

            implement_ops! {
                $name, Sub, sub, SubAssign, sub_assign
            }

            implement_ops! {
                $name, Mul, mul, MulAssign, mul_assign
            }

            implement_ops! {
                $name, Div, div, DivAssign, div_assign
            }

            implement_ops! {
                $name, Rem, rem, RemAssign, rem_assign
            }

            impl<T: Copy + ops::Neg<Output = T>> ops::Neg for $name<T> {
                type Output = $name<T>;

                fn neg(self) -> $name<T> {
                    let mut index = 0;

                    $(
                        let $field = -self.0[index];
                        index += 1;
                    )*

                    $name([$($field),*])
                }
            } 

            implement_sfold! {
                T: num_traits::Signed, $name, abs
            } 

            implement_sfold! {
                T: Real, $name, ceil
            }

            implement_sfold! {
                T: Real, $name, floor
            }

            implement_sfold! {
                T: Real, $name, round
            } 

            implement_packed! {
                T: PartialEq, $name, packed_eq, bool, |a, b| a == b
            }

            implement_packed! {
                T: PartialEq, $name, packed_ne, bool, |a, b| a != b
            }

            implement_packed! {
                T: PartialOrd, $name, packed_lt, bool, |a, b| a < b
            }

            implement_packed! {
                T: PartialOrd, $name, packed_le, bool, |a, b| a <= b
            }

            implement_packed! {
                T: PartialOrd, $name, packed_gt, bool, |a, b| a > b
            }

            implement_packed! {
                T: PartialOrd, $name, packed_ge, bool, |a, b| a >= b
            } 

            impl $name<bool> {
                /// Is any element true?
                pub(crate) fn any(self) -> bool {
                    let Self([$($field),*]) = self;

                    $(
                        if $field {
                            return true;
                        }
                    )*

                    false
                }

                /// Are all elements true?
                pub(crate) fn all(self) -> bool {
                    let Self([$($field),*]) = self;

                    $(
                        if !$field {
                            return false;
                        }
                    )*

                    true
                }
            }
        }

        pub(crate) use self::$modname::$name;
    }
}

vector_type! {
    /// A two-wide vector.
    Double([a, b]) [2] double
}

vector_type! {
    /// A four-wide vector.
    Quad([x, y, z, w]) [4] quad
}

impl<T> Quad<T> {
    /// Split into two double-wide vectors.
    pub(crate) fn split(self) -> (Double<T>, Double<T>) {
        let Quad([x, y, z, w]) = self;

        (Double([x, y]), Double([z, w]))
    }

    /// Low-order `Double` of this vector.
    pub(crate) fn lo(self) -> Double<T> {
        let Quad([x, y, _, _]) = self;

        Double([x, y])
    }

    /// High-order `Double` of this vector.
    pub(crate) fn hi(self) -> Double<T> {
        let Quad([_, _, z, w]) = self;

        Double([z, w])
    }

    /// Create from two `Double`s.
    pub(crate) fn from_double(a: Double<T>, b: Double<T>) -> Self {
        let Double([x, y]) = a;
        let Double([z, w]) = b;

        Quad([x, y, z, w])
    }
}

impl<T> Double<T> {
    /// Swap the elements of the vector.
    pub(crate) fn swap(self) -> Self {
        let Double([a, b]) = self;

        Double([b, a])
    }
}
