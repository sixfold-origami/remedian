//! An example showcasing calculating the median on a dataset using a custom data type

use remedian::RemedianBlock;

/// Our custom data type: a classification enum
///
/// Note that only [`Clone`] and [`PartialOrd`] are strictly necessary here
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    #[default]
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Some sample data to calculate the median for
///
/// In practice, this will probably be a much larger stream
/// Note that the exact median is [`Class::Medium`]
const DATA: [Class; 15] = [
    Class::Medium,
    Class::Low,
    Class::High,
    Class::Medium,
    Class::Low,
    Class::Medium,
    Class::VeryHigh,
    Class::Medium,
    Class::High,
    Class::Low,
    Class::Medium,
    Class::VeryHigh,
    Class::High,
    Class::High,
    Class::Low,
];

fn main() {
    let mut remedian = RemedianBlock::default();

    // Read data points from our data source, and fold them into the remedian
    // It just works!
    for data_point in DATA {
        remedian.add_sample_point(data_point);
    }

    // Get our (approximate) answer
    let median = remedian.median_or_default();
    println!("Approximated the median as: {median:?}");
}
