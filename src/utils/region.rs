//! Region type for rectangular screen areas.

/// A rectangular region of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    /// X offset (column).
    pub x: i32,
    /// Y offset (row).
    pub y: i32,
    /// Width of the region in cells.
    pub width: usize,
    /// Height of the region in cells.
    pub height: usize,
}

impl Region {
    /// Create a new region.
    pub fn new(x: i32, y: i32, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let region = Region::new(1, 2, 3, 4);
        assert_eq!(region.x, 1);
        assert_eq!(region.y, 2);
        assert_eq!(region.width, 3);
        assert_eq!(region.height, 4);
    }

    #[test]
    fn test_clone() {
        let region = Region::new(10, 20, 30, 40);
        let cloned = region;
        assert_eq!(region, cloned);
    }

    #[test]
    fn test_equality() {
        let a = Region::new(0, 0, 100, 50);
        let b = Region::new(0, 0, 100, 50);
        let c = Region::new(1, 0, 100, 50);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_debug() {
        let region = Region::new(0, 0, 80, 24);
        let debug = format!("{:?}", region);
        assert!(debug.contains("Region"));
        assert!(debug.contains("80"));
        assert!(debug.contains("24"));
    }

    #[test]
    fn test_negative_coordinates() {
        let region = Region::new(-5, -10, 20, 15);
        assert_eq!(region.x, -5);
        assert_eq!(region.y, -10);
    }

    #[test]
    fn test_zero_dimensions() {
        let region = Region::new(0, 0, 0, 0);
        assert_eq!(region.width, 0);
        assert_eq!(region.height, 0);
    }
}
