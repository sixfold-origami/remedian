//! A basic example showing minimal usage
//!
//! We construct a [`RemedianBlock`], fill it with data, and then read out the approximate median

use remedian::RemedianBlock;

/// Some sample data to calculate the median for
///
/// In practice, this will probably be a much larger stream
/// Note that the exact median is 44.5
const DATA: [f32; 15] = [
    18.6, 83.1, 21.5, 21.4, 63.4, 64.1, 4.6, 92.7, 31.1, 94.8, 2.4, 44.5, 70.0, 17.1, 61.0,
];

fn main() {
    // The default block is configured with a reasonable size
    // It can account for roughly 25 billion sample points before running out of space
    // But it stores at most 110 f32's at a time
    let mut remedian = RemedianBlock::default();

    // Read data points from our data source, and fold them into the remedian
    for data_point in DATA {
        remedian.add_sample_point(data_point);
    }

    // Get our (approximate) answer
    let median = remedian.median();
    println!("Approximated the median as: {median}");
}
