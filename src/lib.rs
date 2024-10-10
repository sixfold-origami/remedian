#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::cmp::Ordering;

/// Current remedian state calculated for a data stream with data of type `T`
///
/// The [`Self::new`] constructor creates the block in an initial state.
/// Then, data points can be subsequently added with [`Self::add_sample_point`].
/// The median can then be fetched at any time with [`Self::median`].
///
/// The maximum number of collectable sample points is equal to `remedian_base ^ remedian_exponent`.
/// After this many points have been collected, the block will be **locked**, and [`Self::add_sample_point`] will be a no-op.
#[derive(Debug, Clone)]
pub struct RemedianBlock<T: PartialOrd + Clone> {
    /// Base value to use for calculating the remedian
    ///
    /// This should always be an odd number, as it makes the calculation faster
    remedian_base: usize,
    /// Exponent value to use for calculating the remedian
    remedian_exponent: usize,

    /// Total data points
    count: u64,

    /// A [`Self::remedian_base`]*[`Self::remedian_exponent`] scratch matrix used for calculating the median
    ///
    /// A scratch matrix of this size gives us a sample size of [`Self::remedian_base`]^[`Self::remedian_exponent`]
    remedian_scratch: Vec<Vec<T>>,

    /// Flag for whether the `remedian_scratch` is full
    ///
    /// After it's full, we can't collect any more sample points,
    /// so we shouldn't try to push in any more.
    locked: bool,
}

impl<T: PartialOrd + Clone> Default for RemedianBlock<T> {
    /// Initializes a remedian block with a base value of 11 and an exponent of 10.
    ///
    /// This is a reasonable default for most applications, and provides room for roughly 25 billion sample points.
    fn default() -> Self {
        Self::new(11, 10)
    }
}

impl<T: PartialOrd + Clone> RemedianBlock<T> {
    /// Constructs a new [`Self`], without any sample points collected
    ///
    /// Inputs:
    /// - `remedian_base`: Base value to use for the remedian. Must be odd.
    /// - `remedian_exponent`: Exponent value to use for the remedian.
    ///
    /// See the struct-level docs for more information.
    /// If you are unsure of what to use, [`Self::default`] provides reasonable defaults.
    pub fn new(remedian_base: usize, remedian_exponent: usize) -> Self {
        if remedian_base % 2 == 0 {
            #[cfg(feature = "logging")]
            log::warn!(
                "Got even remedian base: {}. This will result in inaccuracies.",
                remedian_base
            );

            #[cfg(not(feature = "logging"))]
            eprintln!(
                "Got even remedian base: {}. This will result in inaccuracies.",
                remedian_base
            );
        }

        let mut remedian_scratch = Vec::with_capacity(remedian_exponent);
        for _ in 0..remedian_exponent {
            remedian_scratch.push(Vec::with_capacity(remedian_base));
        }

        Self {
            remedian_base,
            remedian_exponent,
            count: 0,
            remedian_scratch,
            locked: false,
        }
    }

    /// Whether the block is currently locked
    ///
    /// Locked blocks cannot collect any more sample points,
    /// so calling [`Self::add_sample_point`] will be a no-op.
    pub fn locked(&self) -> bool {
        self.locked
    }

    /// Total number of sample points collected so far
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Processes a new sample point in the stream, updating the rolling median
    ///
    /// Returns whether the point was actually added
    pub fn add_sample_point(&mut self, sample_point: T) -> bool {
        if !self.locked {
            self.count += 1;
            self.remedian_scratch[0].push(sample_point);

            // Check each batch to see if it's full, carrying intermediate medians to the next batch until
            // we either run out of space, or there's nothing left to carry
            for i in 0..self.remedian_exponent {
                let batch = &mut self.remedian_scratch[i];

                if batch.len() == self.remedian_base {
                    // Batch is full

                    if i == self.remedian_exponent - 1 {
                        // This is the last batch, so there's no where to carry to
                        // Lock the scratch and call it a day

                        batch.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
                        self.locked = true;
                    } else {
                        // Not the last batch yet, so calculate the intermediate median,
                        // carry it to the next batch, and empty the batch

                        batch.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
                        let intermediate_median = batch[self.remedian_base / 2].clone();
                        batch.clear();

                        self.remedian_scratch[i + 1].push(intermediate_median);
                    }
                } else {
                    // Nothing left to ripple carry, so we are done here
                    break;
                }
            }

            true
        } else {
            false
        }
    }

    /// Gets the approxmate median of the data points processed
    ///
    /// If no data has been processed, this returns [`None`].
    /// Otherwise, a value is always returned.
    /// For an unchecked version, use [`Self::median_or_default`].
    pub fn median(&self) -> Option<T> {
        if self.locked {
            // We filled our maximum samples, so just take the median of the final batch
            // Note that it's sorted in `add_sample_point` above
            Some(self.remedian_scratch[self.remedian_exponent - 1][self.remedian_base / 2].clone())
        } else {
            // Not all the batches are full, so calculate a weighted median based on what we have

            let mut weighted_values = Vec::new();
            for (i, batch) in self.remedian_scratch.iter().enumerate() {
                for m in batch.iter() {
                    weighted_values.push((m, (self.remedian_base as u64).pow(i as u32)));
                }
            }

            weighted_values.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap_or(Ordering::Equal));

            let mut running_weight = 0;
            for (m, w) in weighted_values.into_iter() {
                running_weight += w;
                if running_weight >= (self.count / 2) {
                    return Some(m.clone());
                }
            }

            // Degenerate case where no data has been processed
            // Return None
            None
        }
    }
}

impl<T: PartialOrd + Clone + Default> RemedianBlock<T> {
    /// Gets the approxmate median of the data points processed
    ///
    /// If no data has been processed, this returns `T::default()`.
    /// Equivalent to calling [`Self::median()`]`.unwrap_or_default()`
    #[inline]
    pub fn median_or_default(&self) -> T {
        self.median().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use super::*;

    /// Expected median value for the 2000_values test dataset
    const EXPECTED_MEDIAN: f32 = 500.5;

    /// The approximated median must be within this value of the [`EXPECTED_MEDIAN`] to be considered correct
    const MEDIAN_ERROR_LIMIT: f32 = 3.0;

    fn load_test_data() -> Vec<f32> {
        let mut data = Vec::with_capacity(2000);
        let f = BufReader::new(File::open("./test_data/2000_values.txt").unwrap());

        for line in f.lines() {
            let v: f32 = line.unwrap().parse().unwrap();
            data.push(v);
        }

        data
    }

    #[test]
    fn median_not_full() {
        let mut remedian = RemedianBlock::default();

        for v in load_test_data().into_iter() {
            remedian.add_sample_point(v);
        }

        assert!((remedian.median_or_default() - EXPECTED_MEDIAN).abs() < MEDIAN_ERROR_LIMIT);
    }

    #[test]
    fn median_full() {
        let mut remedian = RemedianBlock::new(11, 3);

        for v in load_test_data().into_iter() {
            remedian.add_sample_point(v);
        }

        assert!((remedian.median_or_default() - EXPECTED_MEDIAN).abs() < MEDIAN_ERROR_LIMIT);
    }

    #[test]
    fn locked_not_full() {
        let mut remedian = RemedianBlock::default();

        for v in load_test_data().into_iter() {
            remedian.add_sample_point(v);
        }

        assert!(!remedian.locked());
    }

    #[test]
    fn locked_full() {
        let mut remedian = RemedianBlock::new(11, 3);

        for v in load_test_data().into_iter() {
            remedian.add_sample_point(v);
        }

        assert!(remedian.locked());
    }

    #[test]
    fn count_not_full() {
        let mut remedian = RemedianBlock::default();

        for v in load_test_data().into_iter() {
            remedian.add_sample_point(v);
        }

        assert_eq!(remedian.count(), 2000);
    }

    #[test]
    fn count_full() {
        let mut remedian = RemedianBlock::new(11, 3);

        for v in load_test_data().into_iter() {
            remedian.add_sample_point(v);
        }

        assert_eq!(remedian.count(), 1331);
    }

    #[test]
    fn no_data() {
        let remedian = RemedianBlock::<f32>::default();
        assert_eq!(remedian.median(), None);
        assert_eq!(remedian.count(), 0);
        assert!(!remedian.locked())
    }

    #[test]
    fn one_data() {
        let mut remedian = RemedianBlock::default();
        remedian.add_sample_point(10.);

        assert_eq!(remedian.median(), Some(10.));
        assert_eq!(remedian.count(), 1);
        assert!(!remedian.locked())
    }
}
