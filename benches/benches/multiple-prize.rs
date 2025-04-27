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
    config = Criterion::default().noise_threshold(0.03);
    targets = multiple_large_t_large_p, multiple_large_t_medium_p, multiple_large_t_small_p, multiple_medium_t_large_p, multiple_medium_t_medium_p, multiple_medium_t_medium_p_csprng, multiple_medium_t_small_p
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

const NUM_ENTRANTS: usize = 65536;
const FACTOR: usize = 3;
const TIME_MEASUREMENT_BASE: Duration = Duration::from_secs(10);
const WARM_UP_COEFF: u32 = 5;

// `large` would be k * n * log2(n), where k is `FACTOR`
// `medium` would be k * n
// `small` would be k * log2(n)
const fn large() -> usize {
    FACTOR * NUM_ENTRANTS * NUM_ENTRANTS.ilog2() as usize
}
const fn medium() -> usize {
    FACTOR * NUM_ENTRANTS
}
const fn small() -> usize {
    FACTOR * NUM_ENTRANTS.ilog2() as usize
}

// least time needed for measuring 100 samples
fn measure(time_coeff: u32) -> Duration {
    time_coeff * TIME_MEASUREMENT_BASE
}

// the warmup would take at least 20 cycles
// for at least 100 samples, the coeff is 100/20 = 5
fn warm(time_coeff: u32) -> Duration {
    time_coeff * TIME_MEASUREMENT_BASE / WARM_UP_COEFF
}

// bench function naming rule
// multiple_[number of tickets_t]_[number of prizes_p]_[Rng]
// Rng would be either Prng (default) or CsPrng
pub fn multiple_large_t_large_p(c: &mut Criterion) {
    let mut g = c.benchmark_group("multiple_large_t_large_p");
    let time_coeff = 9;
    let nr_ticket = large();
    let nr_prize = large();
    g.measurement_time(measure(time_coeff)).warm_up_time(warm(time_coeff));
    #[cfg(feature = "array")]
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

    #[cfg(feature = "tree")]
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

pub fn multiple_large_t_medium_p(c: &mut Criterion) {
    let mut g = c.benchmark_group("multiple_large_t_medium_p");
    let nr_ticket = large();
    let nr_prize = medium();
    let time_coeff = 2;
    g.measurement_time(measure(time_coeff)).warm_up_time(warm(time_coeff));
    #[cfg(feature = "array")]
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

    #[cfg(feature = "tree")]
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

pub fn multiple_large_t_small_p(c: &mut Criterion) {
    let mut g = c.benchmark_group("multiple_large_t_small_p");
    let nr_ticket = large();
    let nr_prize = small();
    let time_coeff = 1;
    g.measurement_time(measure(time_coeff)).warm_up_time(warm(time_coeff));
    #[cfg(feature = "array")]
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

    #[cfg(feature = "tree")]
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

pub fn multiple_medium_t_large_p(c: &mut Criterion) {
    let mut g = c.benchmark_group("multiple_medium_t_large_p");
    let nr_ticket = medium();
    let nr_prize = large();
    let time_coeff = 4;
    g.measurement_time(measure(time_coeff)).warm_up_time(warm(time_coeff));
    #[cfg(feature = "array")]
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

    #[cfg(feature = "tree")]
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

pub fn multiple_medium_t_medium_p(c: &mut Criterion) {
    let mut g = c.benchmark_group("multiple_medium_t_medium_p");
    let nr_ticket = medium();
    let nr_prize = medium();
    let time_coeff = 1;
    g.measurement_time(measure(time_coeff)).warm_up_time(warm(time_coeff));
    #[cfg(feature = "array")]
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

    #[cfg(feature = "tree")]
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

pub fn multiple_medium_t_medium_p_csprng(c: &mut Criterion) {
    let mut g = c.benchmark_group("multiple_medium_t_medium_p_csprng");
    let nr_ticket = medium();
    let nr_prize = medium();
    let time_coeff = 1;
    g.measurement_time(measure(time_coeff)).warm_up_time(warm(time_coeff));
    #[cfg(feature = "array")]
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

    #[cfg(feature = "tree")]
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

pub fn multiple_medium_t_small_p(c: &mut Criterion) {
    let mut g = c.benchmark_group("multiple_medium_t_small_p");
    let nr_ticket = medium();
    let nr_prize = small();
    let time_coeff = 1;
    g.measurement_time(measure(time_coeff)).warm_up_time(warm(time_coeff));
    #[cfg(feature = "array")]
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

    #[cfg(feature = "tree")]
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
