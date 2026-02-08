//! Measurement module for tracking minimum and maximum rendering widths.
//!
//! Rust port of Python's `rich/measure.py`.

use std::fmt;
use std::ops::Add;

/// Stores the minimum and maximum widths (in cells) required to render an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Measurement {
    pub minimum: usize,
    pub maximum: usize,
}

impl Measurement {
    /// Create a new `Measurement` with the given minimum and maximum widths.
    pub fn new(minimum: usize, maximum: usize) -> Self {
        Self { minimum, maximum }
    }

    /// The difference between the maximum and minimum widths.
    pub fn span(&self) -> usize {
        self.maximum.saturating_sub(self.minimum)
    }

    /// Normalize the measurement so that minimum is never greater than maximum.
    pub fn normalize(&self) -> Measurement {
        let min = self.minimum.min(self.maximum);
        Measurement {
            minimum: min,
            maximum: self.maximum.max(min),
        }
    }

    /// Clamp the maximum width to at most `width`, clamping minimum as well if needed.
    pub fn with_maximum(&self, width: usize) -> Measurement {
        Measurement {
            minimum: self.minimum.min(width),
            maximum: self.maximum.min(width),
        }
    }

    /// Ensure both minimum and maximum are at least `width`.
    pub fn with_minimum(&self, width: usize) -> Measurement {
        Measurement {
            minimum: self.minimum.max(width),
            maximum: self.maximum.max(width),
        }
    }

    /// Apply optional minimum and maximum width constraints.
    pub fn clamp(
        &self,
        min_width: Option<usize>,
        max_width: Option<usize>,
    ) -> Measurement {
        let mut m = *self;
        if let Some(min_w) = min_width {
            m = m.with_minimum(min_w);
        }
        if let Some(max_w) = max_width {
            m = m.with_maximum(max_w);
        }
        m
    }
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Measurement({}, {})", self.minimum, self.maximum)
    }
}

impl Add for Measurement {
    type Output = Measurement;

    /// Combine two measurements by taking the maximum of each field.
    fn add(self, rhs: Self) -> Self::Output {
        Measurement {
            minimum: self.minimum.max(rhs.minimum),
            maximum: self.maximum.max(rhs.maximum),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = Measurement::new(10, 100);
        assert_eq!(m.minimum, 10);
        assert_eq!(m.maximum, 100);
    }

    #[test]
    fn test_span() {
        assert_eq!(Measurement::new(10, 100).span(), 90);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(
            Measurement::new(20, 100).clamp(Some(10), Some(50)),
            Measurement::new(20, 50)
        );
        assert_eq!(
            Measurement::new(20, 100).clamp(Some(30), Some(50)),
            Measurement::new(30, 50)
        );
        assert_eq!(
            Measurement::new(20, 100).clamp(None, Some(50)),
            Measurement::new(20, 50)
        );
        assert_eq!(
            Measurement::new(20, 100).clamp(Some(30), None),
            Measurement::new(30, 100)
        );
        assert_eq!(
            Measurement::new(20, 100).clamp(None, None),
            Measurement::new(20, 100)
        );
    }

    #[test]
    fn test_normalize() {
        assert_eq!(
            Measurement::new(100, 50).normalize(),
            Measurement::new(50, 50)
        );
    }

    #[test]
    fn test_with_maximum() {
        assert_eq!(
            Measurement::new(10, 100).with_maximum(50),
            Measurement::new(10, 50)
        );
    }

    #[test]
    fn test_with_minimum() {
        assert_eq!(
            Measurement::new(10, 100).with_minimum(50),
            Measurement::new(50, 100)
        );
    }

    #[test]
    fn test_display() {
        let m = Measurement::new(10, 100);
        assert_eq!(format!("{m}"), "Measurement(10, 100)");
    }

    #[test]
    fn test_add() {
        let a = Measurement::new(10, 50);
        let b = Measurement::new(20, 40);
        let result = a + b;
        assert_eq!(result, Measurement::new(20, 50));
    }
}
