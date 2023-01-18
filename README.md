# blood-geometry

`blood-geometry` is a toolkit that provides a wide variety of two-dimensional geometric primitives. The goal is for this project to be a "one-stop shop" for geometry-related functionality.

Out of the box, `blood-geometry` provides the following:

* Point, vector, rectangle and size types.
* Quadratic and cubic bezier curves.
* A variety of shape types.
* Traits for dealing with paths and shapes.
* Matrix transforms.

With the `alloc` feature enabled, `blood-geometry` also provides ways to rasterize shapes and paths.

## Third-Party Code Notice

Code for this project is partially derived from the [`kurbo`] project (dual licensed under the MIT License or the Apache License 2.0), the [`lyon`] project (dual licensed under the MIT License or the Apache License 2.0), and [`cairo`] (dual licensed under the GNU Lesser General Public License version 2.1 or the Mozilla Public License version 1.1).

[`kurbo`]: https://crates.io/crates/kurbo
[`lyon`]: https://crates.io/crates/lyon
[`cairo`]: https://www.cairographics.org/

## License

Copyright 2023 John Nunley

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>. 
