//! Algorithms for distributing integers proportionally.
//!
//! Port of Python `rich/_ratio.py`.

/// Trait for items that participate in proportional distribution.
///
/// Each edge can have a fixed `size`, a `ratio` (flex weight), and a `minimum_size`.
pub trait Edge {
    /// Fixed size, or `None` if this edge is flexible.
    fn size(&self) -> Option<usize>;

    /// Flex weight for proportional distribution. Default is 1.
    fn ratio(&self) -> usize {
        1
    }

    /// Minimum size for flexible edges. Default is 1.
    fn minimum_size(&self) -> usize {
        1
    }
}

/// Divide `total` space among edges with optional fixed size, ratio, and minimum_size.
///
/// Edges with a fixed `size` get exactly that. Remaining space is distributed
/// proportionally by `ratio` among flexible edges, respecting `minimum_size`.
///
/// Uses iterative approach: finds edges below minimum, fixes them, repeats.
pub fn ratio_resolve(total: usize, edges: &[impl Edge]) -> Vec<usize> {
    // sizes[i] = Some(n) means fixed/resolved, None means still flexible
    let mut sizes: Vec<Option<usize>> = edges.iter().map(|e| e.size()).collect();

    loop {
        // If no None entries remain, we're done
        if sizes.iter().all(|s| s.is_some()) {
            break;
        }

        // Collect flexible (unresolved) edges
        let flexible: Vec<(usize, &dyn Edge)> = sizes
            .iter()
            .zip(edges.iter())
            .enumerate()
            .filter(|(_, (size, _))| size.is_none())
            .map(|(i, (_, edge))| (i, edge as &dyn Edge))
            .collect();

        // Calculate remaining space
        let used: usize = sizes.iter().filter_map(|s| *s).sum();
        let remaining = total.saturating_sub(used);

        if remaining == 0 {
            // No space left: assign minimums to all flexible edges
            for (i, edge) in &flexible {
                sizes[*i] = Some(edge.minimum_size());
            }
            break;
        }

        // total_ratio = sum of ratios across flexible edges
        let total_ratio: usize = flexible.iter().map(|(_, e)| e.ratio()).sum();

        if total_ratio == 0 {
            // All ratios are zero; assign minimums
            for (i, edge) in &flexible {
                sizes[*i] = Some(edge.minimum_size());
            }
            break;
        }

        // Check if any flexible edge would get less than its minimum.
        // Comparison: portion * edge.ratio <= edge.minimum_size
        // portion = remaining / total_ratio
        // So: remaining * edge.ratio <= edge.minimum_size * total_ratio
        let mut found_below_minimum = false;
        for &(index, edge) in &flexible {
            if remaining * edge.ratio() <= edge.minimum_size() * total_ratio {
                sizes[index] = Some(edge.minimum_size());
                found_below_minimum = true;
                break;
            }
        }

        if !found_below_minimum {
            // All flexible edges fit above minimum. Distribute with remainder tracking.
            // portion = remaining / total_ratio (as fraction)
            // For each edge: size = floor((portion * edge.ratio + accumulated_remainder))
            //                remainder = (portion * edge.ratio + accumulated_remainder) - size
            //
            // Using integer math:
            //   accumulated as numerator over total_ratio
            //   portion * edge.ratio = remaining * edge.ratio / total_ratio
            //
            //   numerator = remaining * edge.ratio + remainder_num
            //   size = numerator / total_ratio
            //   remainder_num = numerator % total_ratio

            let mut remainder_num: usize = 0;
            for (index, edge) in flexible {
                let numerator = remaining * edge.ratio() + remainder_num;
                let size = numerator / total_ratio;
                remainder_num = numerator % total_ratio;
                sizes[index] = Some(size);
            }
            break;
        }
        // If we found one below minimum, loop again with it fixed
    }

    sizes.into_iter().map(|s| s.unwrap_or(0)).collect()
}

/// Reduce `values` by distributing `total` reduction proportionally according to `ratios`,
/// capped by `maximums`.
///
/// Ratios are zeroed for entries where the corresponding maximum is 0.
pub fn ratio_reduce(
    total: usize,
    ratios: &[usize],
    maximums: &[usize],
    values: &[usize],
) -> Vec<usize> {
    // Zero out ratios where maximum is 0
    let ratios: Vec<usize> = ratios
        .iter()
        .zip(maximums.iter())
        .map(|(&r, &m)| if m > 0 { r } else { 0 })
        .collect();

    let mut total_ratio: usize = ratios.iter().sum();
    if total_ratio == 0 {
        return values.to_vec();
    }

    let mut total_remaining = total;
    let mut result = Vec::with_capacity(values.len());

    for ((&ratio, &maximum), &value) in ratios.iter().zip(maximums.iter()).zip(values.iter()) {
        if ratio > 0 && total_ratio > 0 {
            // distributed = min(maximum, round(ratio * total_remaining / total_ratio))
            let distributed = {
                let product = ratio * total_remaining;
                // Manual rounding: (product * 2 + total_ratio) / (2 * total_ratio)
                let rounded = (product * 2 + total_ratio) / (2 * total_ratio);
                rounded.min(maximum)
            };
            result.push(value.saturating_sub(distributed));
            total_remaining = total_remaining.saturating_sub(distributed);
            total_ratio -= ratio;
        } else {
            result.push(value);
        }
    }

    result
}

/// Distribute `total` across entries proportionally by `ratios`, with optional `minimums`.
///
/// If `minimums` is provided, ratios are zeroed for entries with zero minimum.
/// Guarantees the sum of results equals `total` (may over-assign to the last entry
/// to handle rounding).
pub fn ratio_distribute(total: usize, ratios: &[usize], minimums: Option<&[usize]>) -> Vec<usize> {
    let ratios: Vec<usize> = if let Some(mins) = minimums {
        ratios
            .iter()
            .zip(mins.iter())
            .map(|(&r, &m)| if m > 0 { r } else { 0 })
            .collect()
    } else {
        ratios.to_vec()
    };

    let mut total_ratio: usize = ratios.iter().sum();
    assert!(total_ratio > 0, "Sum of ratios must be > 0");

    let mut total_remaining = total;
    let default_minimums: Vec<usize>;
    let mins: &[usize] = if let Some(m) = minimums {
        m
    } else {
        default_minimums = vec![0; ratios.len()];
        &default_minimums
    };

    let mut result = Vec::with_capacity(ratios.len());

    for (&ratio, &minimum) in ratios.iter().zip(mins.iter()) {
        let distributed = if total_ratio > 0 {
            // ceil(ratio * total_remaining / total_ratio)
            let product = ratio * total_remaining;
            let ceiled = product.div_ceil(total_ratio);
            ceiled.max(minimum)
        } else {
            total_remaining
        };
        result.push(distributed);
        total_ratio = total_ratio.saturating_sub(ratio);
        total_remaining = total_remaining.saturating_sub(distributed);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple test edge implementation.
    struct TestEdge {
        size: Option<usize>,
        ratio: usize,
        minimum_size: usize,
    }

    impl TestEdge {
        fn new(size: Option<usize>, ratio: usize, minimum_size: usize) -> Self {
            Self {
                size,
                ratio,
                minimum_size,
            }
        }

        fn flexible(ratio: usize) -> Self {
            Self::new(None, ratio, 1)
        }

        fn fixed(size: usize) -> Self {
            Self::new(Some(size), 1, 1)
        }
    }

    impl Edge for TestEdge {
        fn size(&self) -> Option<usize> {
            self.size
        }
        fn ratio(&self) -> usize {
            self.ratio
        }
        fn minimum_size(&self) -> usize {
            self.minimum_size
        }
    }

    // ---- ratio_resolve tests ----

    #[test]
    fn test_resolve_all_fixed() {
        let edges = vec![
            TestEdge::fixed(10),
            TestEdge::fixed(20),
            TestEdge::fixed(30),
        ];
        let result = ratio_resolve(100, &edges);
        assert_eq!(result, vec![10, 20, 30]);
    }

    #[test]
    fn test_resolve_all_flexible_equal_ratio() {
        let edges = vec![
            TestEdge::flexible(1),
            TestEdge::flexible(1),
            TestEdge::flexible(1),
        ];
        let result = ratio_resolve(30, &edges);
        assert_eq!(result, vec![10, 10, 10]);
    }

    #[test]
    fn test_resolve_all_flexible_unequal_ratio() {
        let edges = vec![
            TestEdge::flexible(1),
            TestEdge::flexible(2),
            TestEdge::flexible(1),
        ];
        let result = ratio_resolve(40, &edges);
        assert_eq!(result, vec![10, 20, 10]);
    }

    #[test]
    fn test_resolve_mixed_fixed_and_flexible() {
        let edges = vec![
            TestEdge::fixed(10),
            TestEdge::flexible(1),
            TestEdge::flexible(1),
        ];
        let result = ratio_resolve(30, &edges);
        assert_eq!(result, vec![10, 10, 10]);
    }

    #[test]
    fn test_resolve_minimum_size_constraint() {
        // Two flexible edges with ratio 1:1, but 10 total — minimum_size=8 for one
        // The one with min 8 gets fixed at 8, leaving 2 for the other
        let edges = vec![TestEdge::new(None, 1, 8), TestEdge::new(None, 1, 1)];
        let result = ratio_resolve(10, &edges);
        assert_eq!(result, vec![8, 2]);
    }

    #[test]
    fn test_resolve_zero_remaining() {
        let edges = vec![
            TestEdge::fixed(50),
            TestEdge::fixed(50),
            TestEdge::flexible(1),
        ];
        let result = ratio_resolve(100, &edges);
        // The flexible edge gets minimum_size since no space remains
        assert_eq!(result, vec![50, 50, 1]);
    }

    #[test]
    fn test_resolve_zero_total() {
        let edges = vec![TestEdge::flexible(1), TestEdge::flexible(1)];
        let result = ratio_resolve(0, &edges);
        // No space: flexible edges get their minimums
        assert_eq!(result, vec![1, 1]);
    }

    #[test]
    fn test_resolve_remainder_distribution() {
        // 10 split among 3 equal ratios: 3, 3, 4 (or similar with remainder tracking)
        let edges = vec![
            TestEdge::flexible(1),
            TestEdge::flexible(1),
            TestEdge::flexible(1),
        ];
        let result = ratio_resolve(10, &edges);
        // Sum should be <= total
        let sum: usize = result.iter().sum();
        assert!(sum <= 10);
        // Each should be at least 3
        for &v in &result {
            assert!(v >= 3);
        }
    }

    #[test]
    fn test_resolve_single_flexible() {
        let edges = vec![TestEdge::flexible(1)];
        let result = ratio_resolve(50, &edges);
        assert_eq!(result, vec![50]);
    }

    #[test]
    fn test_resolve_ratio_2_1() {
        let edges = vec![TestEdge::flexible(2), TestEdge::flexible(1)];
        let result = ratio_resolve(30, &edges);
        assert_eq!(result, vec![20, 10]);
    }

    #[test]
    fn test_resolve_fixed_larger_than_total() {
        let edges = vec![
            TestEdge::fixed(60),
            TestEdge::fixed(60),
            TestEdge::flexible(1),
        ];
        let result = ratio_resolve(100, &edges);
        // Fixed take 120 > 100, flexible gets minimum
        assert_eq!(result, vec![60, 60, 1]);
    }

    // ---- ratio_reduce tests ----

    #[test]
    fn test_reduce_normal() {
        let ratios = vec![1, 1];
        let maximums = vec![5, 5];
        let values = vec![10, 10];
        let result = ratio_reduce(6, &ratios, &maximums, &values);
        assert_eq!(result, vec![7, 7]);
    }

    #[test]
    fn test_reduce_zero_ratios() {
        let ratios = vec![0, 0];
        let maximums = vec![5, 5];
        let values = vec![10, 10];
        let result = ratio_reduce(6, &ratios, &maximums, &values);
        // total_ratio is 0, return values unchanged
        assert_eq!(result, vec![10, 10]);
    }

    #[test]
    fn test_reduce_all_zero_maximums() {
        let ratios = vec![1, 1];
        let maximums = vec![0, 0];
        let values = vec![10, 10];
        let result = ratio_reduce(6, &ratios, &maximums, &values);
        // Ratios zeroed out because maximums are 0 => return values unchanged
        assert_eq!(result, vec![10, 10]);
    }

    #[test]
    fn test_reduce_capped_by_maximum() {
        let ratios = vec![1, 1];
        let maximums = vec![2, 10];
        let values = vec![10, 10];
        let result = ratio_reduce(10, &ratios, &maximums, &values);
        // First: min(2, round(1*10/2)) = min(2, 5) = 2 => value 8
        // remaining=8, total_ratio=1
        // Second: min(10, round(1*8/1)) = min(10, 8) = 8 => value 2
        assert_eq!(result, vec![8, 2]);
    }

    #[test]
    fn test_reduce_empty() {
        let result = ratio_reduce(0, &[], &[], &[]);
        assert_eq!(result, Vec::<usize>::new());
    }

    #[test]
    fn test_reduce_single() {
        let result = ratio_reduce(3, &[1], &[5], &[10]);
        assert_eq!(result, vec![7]);
    }

    // ---- ratio_distribute tests ----

    #[test]
    fn test_distribute_equal() {
        let result = ratio_distribute(20, &[1, 1], None);
        assert_eq!(result.iter().sum::<usize>(), 20);
        assert_eq!(result, vec![10, 10]);
    }

    #[test]
    fn test_distribute_unequal() {
        let result = ratio_distribute(30, &[2, 1], None);
        assert_eq!(result.iter().sum::<usize>(), 30);
        assert_eq!(result, vec![20, 10]);
    }

    #[test]
    fn test_distribute_with_minimums() {
        let result = ratio_distribute(20, &[1, 1, 1], Some(&[5, 5, 5]));
        let sum: usize = result.iter().sum();
        assert_eq!(sum, 20);
        for (&v, &m) in result.iter().zip([5, 5, 5].iter()) {
            assert!(v >= m);
        }
    }

    #[test]
    fn test_distribute_minimums_zeroes_ratios() {
        // When minimum is 0, ratio is zeroed
        let result = ratio_distribute(20, &[1, 1, 1], Some(&[0, 5, 5]));
        // First entry has minimum 0, so its ratio becomes 0 => gets 0
        assert_eq!(result[0], 0);
        assert_eq!(result.iter().sum::<usize>(), 20);
    }

    #[test]
    fn test_distribute_sum_equals_total() {
        // Various cases to ensure sum always equals total
        for total in [1, 7, 10, 13, 100, 1000] {
            let result = ratio_distribute(total, &[1, 2, 3], None);
            assert_eq!(
                result.iter().sum::<usize>(),
                total,
                "Failed for total={total}"
            );
        }
    }

    #[test]
    fn test_distribute_single() {
        let result = ratio_distribute(42, &[1], None);
        assert_eq!(result, vec![42]);
    }

    #[test]
    fn test_distribute_three_equal() {
        let result = ratio_distribute(10, &[1, 1, 1], None);
        let sum: usize = result.iter().sum();
        assert_eq!(sum, 10);
        // With ceil, first gets 4, second 3, third 3
        assert_eq!(result, vec![4, 3, 3]);
    }

    #[test]
    #[should_panic(expected = "Sum of ratios must be > 0")]
    fn test_distribute_zero_ratios_panics() {
        ratio_distribute(10, &[0, 0], None);
    }
}
