//! A more fully-fledged example, showcasing custom confgiuration and other methods on [`RemedianBlock`]

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
    // Construct an empty remedian block, with a custom base and exponent
    // These values will result in a block which uses 20 f32's of space, but can handle up to 625 sample points
    let mut remedian = RemedianBlock::new(5, 4);

    // Read data points from our data source, and fold them into the remedian
    for data_point in DATA {
        let was_added = remedian.add_sample_point(data_point);

        // We can check if the point was added after each one
        if was_added {
            println!("Point was added");
        } else {
            println!("Point was not added: remedian is full");
        }

        // We can also check this manually
        if remedian.locked() {
            println!("Remedian is full: no further points can be processed");
        } else {
            println!("Remedian is not full");
        }

        // As we go, we can check the current number of total points processed
        println!("Processed {} data points so far", remedian.count());
    }

    // Once we've processed everything, we can get our answer out
    let median = remedian.median_or_default();
    println!("Approximated the median as: {median}");
}
