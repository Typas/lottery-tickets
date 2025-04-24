use criterion::{Criterion, criterion_group, criterion_main};
use lottery_tickets::{entrant::EntrantBuilder, lottery::Lottery, prize::PrizeBuilder};
use rand::SeedableRng;

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench
);
criterion_main!(benches);

pub fn bench(c: &mut Criterion) {
    const MAX_PRIZE_COUNT: usize = 100;
    const NUM_ENTRANTS: usize = 256;

    {
        let mut arraygrp = c.benchmark_group("array");
        arraygrp.bench_function("all_prize_small_rng", |b| {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
            b.iter(|| {
                let prizes = Vec::from_iter(
                    (1..=MAX_PRIZE_COUNT).map(|x| PrizeBuilder::new().count(x).name(format!("{x}")).build()),
                );
                let entrants = (0..NUM_ENTRANTS)
                    .map(|n| {
                        EntrantBuilder::new()
                            .name(format!("{n}"))
                            .ticket_count(n)
                            .build_multiple()
                    })
                    .collect::<Vec<_>>();
                let mut lottery = Lottery::new();
                lottery.set_entrants(entrants);
                lottery.set_prizes(prizes);
                lottery.shuffle_array(&mut rng);
            });
        });
    }

    {
        let mut treegrp = c.benchmark_group("tree");
        treegrp.bench_function("all_prize_small_rng", |b| {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
            b.iter(|| {
                let prizes = Vec::from_iter(
                    (1..=MAX_PRIZE_COUNT).map(|x| PrizeBuilder::new().count(x).name(format!("{x}")).build()),
                );
                let entrants = (0..NUM_ENTRANTS)
                    .map(|n| {
                        EntrantBuilder::new()
                            .name(format!("{n}"))
                            .ticket_count(n)
                            .build_multiple()
                    })
                    .collect::<Vec<_>>();
                let mut lottery = Lottery::new();
                lottery.set_entrants(entrants);
                lottery.set_prizes(prizes);
                lottery.shuffle_tree(&mut rng);
            });
        });
    }
}
