//! Canonical FNV-1a 64-bit hash helper.
//!
//! A single definition shared by every module that previously had its own
//! copy (style, console, rule, console_export).  Using one place guarantees
//! that the constants — offset basis and prime — are always identical, and
//! that we never accidentally swap XOR and multiply order.
//!
//! # Algorithm
//!
//! FNV-1a 64-bit per [the reference](http://www.isthe.com/chongo/tech/comp/fnv/):
//!
//! ```text
//! hash = FNV_OFFSET_BASIS
//! for each byte b:
//!     hash ^= b
//!     hash = hash × FNV_PRIME
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::utils::hash::fnv1a_64;
//!
//! let h = fnv1a_64(b"hello");
//! ```

/// FNV-1a 64-bit offset basis.
pub(crate) const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;

/// FNV-1a 64-bit prime multiplier.
pub(crate) const FNV_PRIME: u64 = 1_099_511_628_211;

/// Compute the FNV-1a 64-bit hash of a byte slice.
///
/// Uses the standard `xor-then-multiply` order per the FNV-1a spec.
/// Each call site that previously duplicated these constants and the loop
/// is replaced by a call to this function.
///
/// # Examples
///
/// ```rust
/// # // Only tests internal API; not part of the public crate interface.
/// ```
#[inline]
pub(crate) fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Extend an existing FNV-1a 64-bit running hash with more bytes.
///
/// Useful when hashing several independent byte slices one after another
/// without first copying them into a single buffer — identical to calling
/// `fnv1a_64` on their concatenation.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::utils::hash::{FNV_OFFSET, fnv1a_64_extend};
///
/// let mut h = crate::utils::hash::FNV_OFFSET;
/// h = fnv1a_64_extend(h, b"foo");
/// h = fnv1a_64_extend(h, b"bar");
/// // equivalent to fnv1a_64(b"foobar")
/// ```
#[inline]
pub(crate) fn fnv1a_64_extend(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_64_empty_is_offset_basis() {
        // FNV-1a of empty input is the offset basis itself.
        assert_eq!(fnv1a_64(b""), FNV_OFFSET);
    }

    #[test]
    fn fnv1a_64_known_vectors() {
        // Known FNV-1a 64-bit values (computed from the reference implementation).
        assert_eq!(fnv1a_64(b"foobar"), 0x85944171f73967e8);
    }

    #[test]
    fn fnv1a_64_different_inputs_differ() {
        assert_ne!(fnv1a_64(b"hello"), fnv1a_64(b"world"));
    }

    #[test]
    fn fnv1a_64_extend_matches_single_call() {
        let combined = fnv1a_64(b"foobar");
        let mut h = FNV_OFFSET;
        h = fnv1a_64_extend(h, b"foo");
        h = fnv1a_64_extend(h, b"bar");
        assert_eq!(h, combined);
    }

    #[test]
    fn fnv1a_64_extend_from_mid_hash() {
        // Extending a partial hash should equal hashing the full concatenation.
        let a = fnv1a_64(b"prefix");
        let extended = fnv1a_64_extend(a, b"suffix");
        let full = fnv1a_64(b"prefixsuffix");
        assert_eq!(extended, full);
    }
}
