use criterion::{Criterion, criterion_group, criterion_main};

use rand::Rng;
use std::iter::repeat;

fn random_number_generator_baseline(c: &mut Criterion) {
    let mut rng = rand::rng();
    const NUM_INVOKATIONS: usize = 10_000;
    c.bench_function("random_number_generator_baseline", |c| {
        c.iter(|| {
            let random_numbers = Vec::from_iter(repeat(()).take(NUM_INVOKATIONS).map(|_| rng.random_ratio(1, 2)));
            assert!(
                random_numbers
                    .into_iter()
                    .filter(|b| *b)
                    .count()
                    .abs_diff(NUM_INVOKATIONS / 2)
                    < NUM_INVOKATIONS * 3 / 100,
                "Sorry Einstein, God plays dice, spurious fail happens, just retry"
            )
        })
    });
}

criterion_group!(benches, random_number_generator_baseline);
criterion_main!(benches);
