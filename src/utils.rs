use crate::error::InnerGridError;

#[derive(Debug, Clone, Copy)]
pub struct ForCellArgs {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

pub fn for_cell(
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
