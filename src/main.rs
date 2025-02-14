mod kdtree;

use std::time::Instant;

use crate::kdtree::KdTree;

use rand::{distr::Uniform, prelude::*, rng};

fn main() {
    let uniform_distr = Uniform::new(0.0, 10.0).expect("invalid distribution");
    let mut rng = rng();
    let data: Vec<[f32; 3]> = (0..1000000)
        .map(|_| {
            [
                uniform_distr.sample(&mut rng),
                uniform_distr.sample(&mut rng),
                uniform_distr.sample(&mut rng),
            ]
        })
        .collect();

    let now = Instant::now();
    let kdtree = KdTree::construct(data);
    let elapsed = now.elapsed();

    println!("construction: {}ms", elapsed.as_millis());
    println!("root: {:?}", kdtree.root());
}
