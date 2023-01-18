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

//! Regions are used to represent rectilinear regions of space.

use crate::box2d::Box;
use crate::Rect;
use num_traits::Zero;

use core::borrow::Borrow;
use core::iter::{self, FromIterator, FusedIterator};
use core::marker::PhantomData;

/// Represents a structure that can represent a region.
pub trait Region<T: Copy> {
    /// The iterator type returned by `boxes_iter`.
    type Iter: Iterator<Item = Box<T>>;

    /// Get an iterator over the boxes in this region.
    fn boxes_iter(self) -> Self::Iter;
}

impl<T: Copy, Bx: Borrow<Box<T>>, I: IntoIterator<Item = Bx>> Region<T> for I {
    type Iter = BoxIter<T, Bx, I::IntoIter>;

    fn boxes_iter(self) -> Self::Iter {
        BoxIter {
            iter: self.into_iter(),
            _phantom: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct BoxIter<T, Bx, I> {
    iter: I,
    _phantom: PhantomData<(T, Bx)>,
}

impl<T: Copy, Bx: Borrow<Box<T>>, I: Iterator<Item = Bx>> Iterator for BoxIter<T, Bx, I> {
    type Item = Box<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|b| *b.borrow())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, |acc, b| f(acc, *b.borrow()))
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|b| *b.borrow())
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|b| *b.borrow())
    }

    fn collect<B>(self) -> B
    where
        Self: Sized,
        B: FromIterator<Self::Item>,
    {
        self.iter.map(|b| *b.borrow()).collect()
    }
}

impl<T: Copy, Bx: Borrow<Box<T>>, I: FusedIterator<Item = Bx>> FusedIterator for BoxIter<T, Bx, I> {}

impl<T: Copy, Bx: Borrow<Box<T>>, I: ExactSizeIterator<Item = Bx>> ExactSizeIterator
    for BoxIter<T, Bx, I>
{
}

impl<T: Copy, Bx: Borrow<Box<T>>, I: DoubleEndedIterator<Item = Bx>> DoubleEndedIterator
    for BoxIter<T, Bx, I>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|b| *b.borrow())
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|b| *b.borrow())
    }
}

/// A region that is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Empty<T: Copy>(PhantomData<T>);

impl<T: Copy> Empty<T> {
    /// Create a new empty region.
    pub fn new() -> Self {
        Empty(PhantomData)
    }
}

impl<T: Copy> Region<T> for Empty<T> {
    type Iter = iter::Empty<Box<T>>;

    fn boxes_iter(self) -> Self::Iter {
        iter::empty()
    }
}

/// Create a new empty region.
pub fn empty<T: Copy>() -> Empty<T> {
    Empty::new()
}

/// A region that consists of a single box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Single<T: Copy>(Box<T>);

impl<T: Copy> Single<T> {
    /// Create a new single box region.
    pub fn new(box_: Box<T>) -> Self {
        Single(box_)
    }
}

impl<T: Copy> Region<T> for Single<T> {
    type Iter = iter::Once<Box<T>>;

    fn boxes_iter(self) -> Self::Iter {
        iter::once(self.0)
    }
}

/// Create a new single box region.
pub fn single<T: Copy>(box_: Box<T>) -> Single<T> {
    Single::new(box_)
}

/// An adaptor that converts a series of rectangles to a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rects<I> {
    iter: I,
}

impl<T: Copy + Zero, Rct: Borrow<Rect<T>>, I: IntoIterator<Item = Rct>> Region<T> for Rects<I> {
    type Iter = RectsIter<T, Rct, I::IntoIter>;

    fn boxes_iter(self) -> Self::Iter {
        RectsIter {
            iter: self.iter.into_iter(),
            _phantom: PhantomData,
        }
    }
}

#[doc(hidden)]
pub struct RectsIter<T, Rct, I> {
    iter: I,
    _phantom: PhantomData<(T, Rct)>,
}

impl<T: Copy + Zero, Rct: Borrow<Rect<T>>, I: Iterator<Item = Rct>> Iterator
    for RectsIter<T, Rct, I>
{
    type Item = Box<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|r| r.borrow().to_box())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, |acc, r| f(acc, r.borrow().to_box()))
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(|r| r.borrow().to_box())
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|r| r.borrow().to_box())
    }

    fn collect<B>(self) -> B
    where
        Self: Sized,
        B: FromIterator<Self::Item>,
    {
        self.iter.map(|r| r.borrow().to_box()).collect()
    }
}

/// An adaptor that converts a series of rectangles to a region.
pub fn rects<T: Copy + Zero, Rct: Borrow<Rect<T>>, I: IntoIterator<Item = Rct>>(
    iter: I,
) -> Rects<I> {
    Rects { iter }
}
