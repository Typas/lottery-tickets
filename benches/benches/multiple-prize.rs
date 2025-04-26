use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use lottery_tickets::{entrant::EntrantBuilder, lottery::Lottery, prize::PrizeBuilder};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng as CsPrng;
use rand_xoshiro::Xoshiro256PlusPlus as Prng;

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench
);
criterion_main!(benches);

macro_rules! bench_iter_multiple {
    (array $bench_func: ident, $rng: ident, $nr_prize: ident, $nr_per_prize: expr, $ticket_per_user: expr) => {
        bench_iter_multiple!(@inner $bench_func, $rng, shuffle_array, $nr_prize, $nr_per_prize, $ticket_per_user)
    };
    (tree $bench_func: ident, $rng: ident, $nr_prize: ident, $nr_per_prize: expr, $ticket_per_user: expr) => {
        bench_iter_multiple!(@inner $bench_func, $rng, shuffle_tree, $nr_prize, $nr_per_prize, $ticket_per_user)
    };
    (@inner $bench_func: ident, $rng: ident, $shuffle_func: ident, $nr_prize: ident, $nr_per_prize: expr, $ticket_per_user: expr) => {
        $bench_func.iter(|| {
            let prizes = Vec::from_iter(
                (0..$nr_prize).map(|x| PrizeBuilder::new().count($nr_per_prize).name(format!("{x}")).build()),
            );
            let entrants = (0..NUM_ENTRANTS)
                .map(|n| {
                    EntrantBuilder::new()
                        .name(format!("{n}"))
                        .ticket_count($ticket_per_user)
                        .build_multiple()
                })
                .collect::<Vec<_>>();
            let mut lottery = Lottery::new();
            lottery.set_entrants(entrants);
            lottery.set_prizes(prizes);
            lottery.$shuffle_func(&mut $rng);
        });
    };
}

pub fn bench(c: &mut Criterion) {
    // bench function naming rule
    // [number of tickets_t]_[number of prizes_p]_[Rng]
    // `large` would be ku log2(u), where k is `FACTOR`
    // `medium` would be u
    // `small` would be log2(u)
    // Rng would be either SmallRng (default) or StdRng
    const NUM_ENTRANTS: usize = 65536;
    const FACTOR: usize = 3;
    {
        let mut g = c.benchmark_group("multiple_large_t_large_p");
        g.measurement_time(Duration::from_secs(30)).sample_size(50);
        let nr_ticket = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        let nr_prize = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("multiple_large_t_medium_p");
        let nr_ticket = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        let nr_prize = NUM_ENTRANTS;
        g.measurement_time(Duration::from_secs(10));
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("multiple_large_t_small_p");
        let nr_ticket = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        let nr_prize = NUM_ENTRANTS.ilog2() as usize;
        g.measurement_time(Duration::from_secs(10));
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("multiple_medium_t_large_p");
        let nr_ticket = NUM_ENTRANTS;
        let nr_prize = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("multiple_medium_t_medium_p");
        let nr_ticket = NUM_ENTRANTS;
        let nr_prize = NUM_ENTRANTS;
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("multiple_medium_t_medium_p_std_rng");
        let nr_ticket = NUM_ENTRANTS;
        let nr_prize = NUM_ENTRANTS;
        g.bench_function("array", |b| {
            let mut rng = CsPrng::seed_from_u64(1);
            bench_iter_multiple!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = CsPrng::seed_from_u64(1);
            bench_iter_multiple!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("multiple_medium_t_small_p");
        let nr_ticket = NUM_ENTRANTS;
        let nr_prize = NUM_ENTRANTS.ilog2() as usize;
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_multiple!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("multiple_medium_t_small_p_std_rng");
        let nr_ticket = NUM_ENTRANTS;
        let nr_prize = NUM_ENTRANTS.ilog2() as usize;
        g.bench_function("array", |b| {
            let mut rng = CsPrng::seed_from_u64(1);
            bench_iter_multiple!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = CsPrng::seed_from_u64(1);
            bench_iter_multiple!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
}
