// use std::time::Duration;
use std::fmt::Debug;

use criterion::*;
use rand::prelude::random;
use rand_distr::{Geometric, Distribution};

use gtree_experiments::{*, klist::*};

fn create_gtree<S: NonemptySet<Item = u32> + Debug>(items: &[(u32, u8 /*rank*/)]) -> GTree<S> {
    let mut t = GTree::Empty;

    for (item, rank) in items {
        t = insert(&t, *item, *rank);
    }

    return t;
}

fn random_gtree_of_size<S: NonemptySet<Item = u32> + Debug>(n: usize, target_node_size: usize) -> GTree<S> {
    let mut items = vec![];
    let geo = Geometric::new(1.0 - (1.0 / ((target_node_size + 1) as f64))).unwrap();

    for _ in 0..n {
        let key: u32 = random();
        let rank = geo.sample(&mut rand::thread_rng()) as u8;

        items.push((key, rank));
    }

    return create_gtree(&items[..]);
}

fn setup<S: NonemptySet<Item = u32> + Debug>(n: usize, target_node_size: usize) -> (GTree<S>, Vec<u32>/* items to search for*/) {
    let mut items = vec![];
    for _ in 0..100 {
        items.push(random());
    }
    return (random_gtree_of_size(n, target_node_size), items);
}

pub fn bench_search(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut group = c.benchmark_group("Search");
    group.plot_config(plot_config);
    // group.sample_size(1000);
    group.sample_size(20);
    // group.measurement_time(Duration::from_secs(150));

    // for i in [100, 1000, 10000, 100000].iter() {
    for i in [128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536].iter() {
        group.bench_with_input(
            BenchmarkId::new("Search 1-Zip", i),
            i,
            |b, i| {
                b.iter_batched_ref(
                    || setup::<NonemptyReverseKList<1, u32>>(*i, 1),
                    |(tree, items)| {
                        for key in items {
                            has(tree, key);
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
        // group.bench_with_input(
        //     BenchmarkId::new("Search 3-Zip", i),
        //     i,
        //     |b, i| {
        //         b.iter_batched_ref(
        //             || setup::<NonemptyReverseKList<3, u32>>(*i, 3),
        //             |(tree, items)| {
        //                 for key in items {
        //                     has(tree, key);
        //                 }
        //             },
        //             BatchSize::SmallInput,
        //         )
        //     },
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("Search 7-Zip", i),
        //     i,
        //     |b, i| {
        //         b.iter_batched_ref(
        //             || setup::<NonemptyReverseKList<7, u32>>(*i, 7),
        //             |(tree, items)| {
        //                 for key in items {
        //                     has(tree, key);
        //                 }
        //             },
        //             BatchSize::SmallInput,
        //         )
        //     },
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("Search 15-Zip", i),
        //     i,
        //     |b, i| {
        //         b.iter_batched_ref(
        //             || setup::<NonemptyReverseKList<15, u32>>(*i, 15),
        //             |(tree, items)| {
        //                 for key in items {
        //                     has(tree, key);
        //                 }
        //             },
        //             BatchSize::SmallInput,
        //         )
        //     },
        // );
        group.bench_with_input(
            BenchmarkId::new("Search 31-Zip", i),
            i,
            |b, i| {
                b.iter_batched_ref(
                    || setup::<NonemptyReverseKList<31, u32>>(*i, 31),
                    |(tree, items)| {
                        for key in items {
                            has(tree, key);
                        }
                    },
                    BatchSize::SmallInput,
                )
            },
        );
        // group.bench_with_input(
        //     BenchmarkId::new("Search 63-Zip", i),
        //     i,
        //     |b, i| {
        //         b.iter_batched_ref(
        //             || setup::<NonemptyReverseKList<63, u32>>(*i, 63),
        //             |(tree, items)| {
        //                 for key in items {
        //                     has(tree, key);
        //                 }
        //             },
        //             BatchSize::SmallInput,
        //         )
        //     },
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("Search 127-Zip", i),
        //     i,
        //     |b, i| {
        //         b.iter_batched_ref(
        //             || setup::<NonemptyReverseKList<127, u32>>(*i, 127),
        //             |(tree, items)| {
        //                 for key in items {
        //                     has(tree, key);
        //                 }
        //             },
        //             BatchSize::SmallInput,
        //         )
        //     },
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("Search 255-Zip", i),
        //     i,
        //     |b, i| {
        //         b.iter_batched_ref(
        //             || setup::<NonemptyReverseKList<255, u32>>(*i, 255),
        //             |(tree, items)| {
        //                 for key in items {
        //                     has(tree, key);
        //                 }
        //             },
        //             BatchSize::SmallInput,
        //         )
        //     },
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("Search 511-Zip", i),
        //     i,
        //     |b, i| {
        //         b.iter_batched_ref(
        //             || setup::<NonemptyReverseKList<511, u32>>(*i, 511),
        //             |(tree, items)| {
        //                 for key in items {
        //                     has(tree, key);
        //                 }
        //             },
        //             BatchSize::SmallInput,
        //         )
        //     },
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("Search 1023-Zip", i),
        //     i,
        //     |b, i| {
        //         b.iter_batched_ref(
        //             || setup::<NonemptyReverseKList<1023, u32>>(*i, 1023),
        //             |(tree, items)| {
        //                 for key in items {
        //                     has(tree, key);
        //                 }
        //             },
        //             BatchSize::SmallInput,
        //         )
        //     },
        // );
    }
    group.finish();
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
