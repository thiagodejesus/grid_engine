// Copyright (c) 2025 Thiago Ramos
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Utility functions for grid operations and cell iteration.
//!
//! This module provides helper functions and structures for working with grid cells,
//! particularly for iterating over rectangular regions within the grid.

use crate::error::InnerGridError;

/// Arguments for iterating over a rectangular region of cells.
///
/// This struct defines a rectangular area in the grid by specifying:
/// - The top-left corner position (x, y)
/// - The width and height of the rectangle
#[derive(Debug, Clone, Copy)]
pub struct ForCellArgs {
    /// X coordinate of the top-left corner
    pub x: usize,
    /// Y coordinate of the top-left corner
    pub y: usize,
    /// Width of the rectangular region
    pub w: usize,
    /// Height of the rectangular region
    pub h: usize,
}

/// Iterates over cells in a rectangular region, executing a callback for each cell.
///
/// This function visits each cell in the specified rectangular region in row-major order
/// (left to right, top to bottom) and executes the provided callback for each cell.
///
/// # Arguments
///
/// * `args` - Defines the rectangular region to iterate over
/// * `callback` - Function to execute for each cell, receiving x and y coordinates
///
/// # Returns
///
/// * `Ok(())` if all cells were processed successfully
/// * `Err(InnerGridError)` if the callback returns an error for any cell
///
/// # Error Handling
///
/// If the callback returns an error for any cell, iteration stops immediately
/// and the error is propagated to the caller.
pub(crate) fn for_cell(
    args: ForCellArgs,
    callback: &mut impl FnMut(usize, usize) -> Result<(), InnerGridError>,
) -> Result<(), InnerGridError> {
    for x in args.x..args.x + args.w {
        for y in args.y..args.y + args.h {
            callback(x, y)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_cell_visits_all_cells() {
        let mut visited = vec![];
        let mut callback = |x, y| {
            visited.push((x, y));
            Ok(())
        };

        let args = ForCellArgs {
            x: 1,
            y: 2,
            w: 2,
            h: 2,
        };
        for_cell(args, &mut callback).unwrap();

        assert_eq!(
            visited,
            vec![(1, 2), (1, 3), (2, 2), (2, 3)],
            "Should visit all cells in the specified rectangle"
        );
    }

    #[test]
    fn test_for_cell_handles_zero_dimensions() {
        let mut callback = |_x, _y| Ok(());

        assert!(for_cell(
            ForCellArgs {
                x: 0,
                y: 0,
                w: 0,
                h: 1
            },
            &mut callback
        )
        .is_ok());

        assert!(for_cell(
            ForCellArgs {
                x: 0,
                y: 0,
                w: 1,
                h: 0
            },
            &mut callback
        )
        .is_ok());

        assert!(for_cell(
            ForCellArgs {
                x: 0,
                y: 0,
                w: 0,
                h: 0
            },
            &mut callback
        )
        .is_ok());
    }

    #[test]
    fn test_for_cell_propagates_error() {
        let mut callback = |x, _y| {
            if x > 1 {
                Err(InnerGridError::OutOfBoundsAccess { x: 0, y: 0 })
            } else {
                Ok(())
            }
        };

        assert!(for_cell(
            ForCellArgs {
                x: 1,
                y: 1,
                w: 2,
                h: 1
            },
            &mut callback
        )
        .is_err());
    }
}
