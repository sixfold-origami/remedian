use std::{
    fs::File,
    io::{BufRead, BufReader},
    time::Duration,
};

use criterion::{criterion_group, criterion_main, Criterion};
use remedian::RemedianBlock;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut data = Vec::with_capacity(2000);
    let f = BufReader::new(File::open("./test_data/2000_values.txt").unwrap());

    for line in f.lines() {
        let v: f32 = line.unwrap().parse().unwrap();
        data.push(v);
    }

    let mut group = c.benchmark_group("benches");
    group
        .measurement_time(Duration::from_secs_f32(10.))
        .sample_size(1000);

    group.bench_function("remedian not full", |b| {
        b.iter(|| {
            let mut remedian = RemedianBlock::default();

            for v in data.iter() {
                remedian.add_sample_point(*v);
            }

            let _median = remedian.median_or_default();
        })
    });

    group.bench_function("remedian full", |b| {
        b.iter(|| {
            let mut remedian = RemedianBlock::new(11, 3);

            for v in data.iter() {
                if !remedian.add_sample_point(*v) {
                    break;
                };
            }

            let _median = remedian.median_or_default();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
