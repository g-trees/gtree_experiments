// use std::time::Duration;
use std::fmt::Debug;

use criterion::*;
use rand::prelude::random;
use rand_distr::{Geometric, Distribution};

use gtree_experiments::{*, klist::*};

fn create_gtree<S: NonemptySet<Item = u64> + Debug>(items: &[(u64, u8 /*rank*/)]) -> GTree<S> {
    let mut t = GTree::Empty;

    for (item, rank) in items {
        t = insert(&t, *item, *rank);
    }

    return t;
}

fn random_gtree_of_size<S: NonemptySet<Item = u64> + Debug>(n: usize, target_node_size: usize) -> GTree<S> {
    let mut items = vec![];
    let geo = Geometric::new(1.0 - (1.0 / (target_node_size as f64))).unwrap();

    for _ in 0..n {
        let key: u64 = random();
        let rank = geo.sample(&mut rand::thread_rng()) as u8;

        items.push((key, rank));
    }

    return create_gtree(&items[..]);
}

fn setup<S: NonemptySet<Item = u64> + Debug>(n: usize, target_node_size: usize) -> (GTree<S>, u64/* item to search for*/) {
    let key: u64 = random();
    return (random_gtree_of_size(n, target_node_size), key);
}

pub fn bench_search(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("Search");
    group.plot_config(plot_config);
    // group.sample_size(1000);
    group.sample_size(100);
    // group.measurement_time(Duration::from_secs(150));

    // for i in [100, 1000, 10000, 100000].iter() {
    for i in [10, 100, 1000, 4000].iter() {
        group.bench_with_input(
            BenchmarkId::new("Search 1-Zip", i),
            i,
            |b, i| {
                b.iter_batched_ref(
                    || setup::<NonemptyReverseKList<1, u64>>(*i, 1),
                    |(tree, key)| has(tree, key),
                    BatchSize::SmallInput,
                )
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Search 8-Zip", i),
            i,
            |b, i| {
                b.iter_batched_ref(
                    || setup::<NonemptyReverseKList<8, u64>>(*i, 8),
                    |(tree, key)| has(tree, key),
                    BatchSize::SmallInput,
                )
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Search 16-Zip", i),
            i,
            |b, i| {
                b.iter_batched_ref(
                    || setup::<NonemptyReverseKList<16, u64>>(*i, 16),
                    |(tree, key)| has(tree, key),
                    BatchSize::SmallInput,
                )
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Search 32-Zip", i),
            i,
            |b, i| {
                b.iter_batched_ref(
                    || setup::<NonemptyReverseKList<32, u64>>(*i, 32),
                    |(tree, key)| has(tree, key),
                    BatchSize::SmallInput,
                )
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Search 64-Zip", i),
            i,
            |b, i| {
                b.iter_batched_ref(
                    || setup::<NonemptyReverseKList<64, u64>>(*i, 64),
                    |(tree, key)| has(tree, key),
                    BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
