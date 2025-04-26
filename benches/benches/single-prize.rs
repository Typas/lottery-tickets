use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use lottery_tickets::{entrant::EntrantBuilder, lottery::Lottery, prize::PrizeBuilder};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng as CsPrng;
use rand_xoshiro::Xoshiro256PlusPlus as Prng;

#[global_allocator]
static ALLOCATOR: jemallocator::Jemalloc = jemallocator::Jemalloc;

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = bench
);
criterion_main!(benches);

macro_rules! bench_iter_single {
    (array $bench_func: ident, $rng: ident, $nr_prize: ident, $nr_per_prize: expr, $ticket_per_user: expr) => {
        bench_iter_single!(@inner $bench_func, $rng, shuffle_array, $nr_prize, $nr_per_prize, $ticket_per_user)
    };
    (tree $bench_func: ident, $rng: ident, $nr_prize: ident, $nr_per_prize: expr, $ticket_per_user: expr) => {
        bench_iter_single!(@inner $bench_func, $rng, shuffle_tree, $nr_prize, $nr_per_prize, $ticket_per_user)
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
                        .build_single()
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
    const TIME_MEASUREMENT_BASE: Duration = Duration::from_secs(10);
    const WARM_UP_COEFF: u32 = 10;
    {
        let mut g = c.benchmark_group("single_large_t_large_p");
        let time_coeff = 18;
        g.measurement_time(time_coeff * TIME_MEASUREMENT_BASE)
            .warm_up_time(time_coeff * TIME_MEASUREMENT_BASE / WARM_UP_COEFF);
        let nr_ticket = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        let nr_prize = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("single_large_t_medium_p");
        let nr_ticket = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        let nr_prize = NUM_ENTRANTS;
        let time_coeff = 9;
        g.measurement_time(time_coeff * TIME_MEASUREMENT_BASE)
            .warm_up_time(time_coeff * TIME_MEASUREMENT_BASE / WARM_UP_COEFF);
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("single_large_t_small_p");
        let nr_ticket = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        let nr_prize = NUM_ENTRANTS.ilog2() as usize;
        let time_coeff = 3;
        g.measurement_time(time_coeff * TIME_MEASUREMENT_BASE)
            .warm_up_time(time_coeff * TIME_MEASUREMENT_BASE / WARM_UP_COEFF);
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("single_medium_t_large_p");
        let nr_ticket = NUM_ENTRANTS;
        let nr_prize = FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize;
        let time_coeff = 6;
        g.measurement_time(time_coeff * TIME_MEASUREMENT_BASE)
            .warm_up_time(time_coeff * TIME_MEASUREMENT_BASE / WARM_UP_COEFF);
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("single_medium_t_medium_p");
        let nr_ticket = NUM_ENTRANTS;
        let nr_prize = NUM_ENTRANTS;
        let time_coeff = 3;
        g.measurement_time(time_coeff * TIME_MEASUREMENT_BASE)
            .warm_up_time(time_coeff * TIME_MEASUREMENT_BASE / WARM_UP_COEFF);
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("single_medium_t_medium_p_std_rng");
        let nr_ticket = NUM_ENTRANTS;
        let nr_prize = NUM_ENTRANTS;
        let time_coeff = 3;
        g.measurement_time(time_coeff * TIME_MEASUREMENT_BASE)
            .warm_up_time(time_coeff * TIME_MEASUREMENT_BASE / WARM_UP_COEFF);
        g.bench_function("array", |b| {
            let mut rng = CsPrng::seed_from_u64(1);
            bench_iter_single!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = CsPrng::seed_from_u64(1);
            bench_iter_single!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
    {
        let mut g = c.benchmark_group("single_medium_t_small_p");
        let nr_ticket = NUM_ENTRANTS;
        let nr_prize = NUM_ENTRANTS.ilog2() as usize;
        let time_coeff = 3;
        g.measurement_time(time_coeff * TIME_MEASUREMENT_BASE)
            .warm_up_time(time_coeff * TIME_MEASUREMENT_BASE / WARM_UP_COEFF);
        g.bench_function("array", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(array
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });

        g.bench_function("tree", |b| {
            let mut rng = Prng::seed_from_u64(1);
            bench_iter_single!(tree
                b,
                rng,
                nr_prize,
                1.max(nr_prize / NUM_ENTRANTS),
                1.max(nr_ticket / NUM_ENTRANTS)
            );
        });
    }
}
