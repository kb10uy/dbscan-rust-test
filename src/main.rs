mod dbscan;
mod kdtree;

use crate::dbscan::dbscan;

use std::time::Instant;

use rand::{distr::Uniform, prelude::*, rng};

fn main() {
    let element_counts = vec![
        10000, 20000, 50000, 80000, 100000, 200000, 300000, 400000, 500000, 800000, 1000000, 5000000, 10000000,
    ];
    for elements in element_counts {
        test_dbscan(elements);
    }
}

fn test_dbscan(elements: usize) {
    let range_scale = (elements as f32).powf(1.0 / 3.0) / 10.0;
    let uniform_distr = Uniform::new(0.0, 10.0 * range_scale).expect("invalid distribution");
    let mut rng = rng();
    let data: Vec<[f32; 3]> = (0..elements)
        .map(|_| {
            [
                uniform_distr.sample(&mut rng),
                uniform_distr.sample(&mut rng),
                uniform_distr.sample(&mut rng),
            ]
        })
        .collect();

    let now = Instant::now();
    let _labels = dbscan(data, 0.05, 6);
    let elapsed = now.elapsed();
    println!("{elements} items: {}us", elapsed.as_micros());
}
