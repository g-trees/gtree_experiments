use std::fmt::Debug;

use rand::prelude::random;
use rand_distr::{Distribution, Geometric, Standard};

use gtree_experiments::{*, klist::*};

fn create_gtree<S: NonemptySet + Debug>(items: &[(S::Item, u8 /*rank*/)]) -> GTree<S> where S::Item: Clone {
    let mut t = GTree::Empty;

    for (item, rank) in items {
        t = insert(&t, item.clone(), *rank);
    }

    return t;
}

fn random_gtree_of_size<S: NonemptySet + Debug>(n: usize, target_node_size: usize) -> GTree<S> where S::Item: Clone, Standard: Distribution<S::Item> {
    let mut items = vec![];
    let geo = Geometric::new(1.0 - (1.0 / ((target_node_size + 1) as f64))).unwrap();

    for _ in 0..n {
        let key: S::Item = random();
        let rank = geo.sample(&mut rand::thread_rng()) as u8;

        items.push((key, rank));
    }

    return create_gtree(&items[..]);
}

fn random_klist_tree<const K: usize, T: Clone + Ord + Debug>(size: usize) -> GTree<NonemptyReverseKList<K, T>> where Standard: Distribution<T> {
    return random_gtree_of_size(size, K);
}

fn repeated_experiment<const K: usize, T: Clone + Ord + Debug>(size: usize, repetitions: usize) where Standard: Distribution<T> {
    let mut results: Vec<(Stats<T>, usize /* physical height */)> = vec![];

    for _ in 0..repetitions {
        let tree: GTree<NonemptyReverseKList<K, T>> = random_klist_tree(size);
        let (stats, _ranks) = gtree_stats(&tree);
        let phy_height = physical_height(&tree);
        results.push((stats, phy_height));
    }

    let perfect_height = (size as f64).log((K + 1) as f64).ceil();

    // Add together all stats, then divide by number of repetitions to obtain averages.

    let mut gnode_height = 0.0f64;
    let mut gnode_count = 0.0f64;
    let mut item_count = 0.0f64;
    let mut item_slot_count = 0.0f64;
    let mut space_amplification = 0.0f64;
    let mut physical_height = 0.0f64;
    let mut height_amplification = 0.0f64;
    let mut average_gnode_size = 0.0f64;

    for (stats, phy_height) in results.iter() {
        gnode_height += stats.gnode_height as f64;
        gnode_count += stats.gnode_count as f64;
        item_count += stats.item_count as f64;
        item_slot_count += stats.item_slot_count as f64;
        space_amplification += (stats.item_slot_count as f64) / (stats.item_count as f64);
        physical_height += *phy_height as f64;
        height_amplification += (*phy_height as f64) / perfect_height;
        average_gnode_size += (item_count as f64) / (gnode_count as f64);
    }

    gnode_height /= repetitions as f64;
    gnode_count /= repetitions as f64;
    item_count /= repetitions as f64;
    item_slot_count /= repetitions as f64;
    space_amplification /= repetitions as f64;
    physical_height /= repetitions as f64;
    height_amplification /= repetitions as f64;
    average_gnode_size /= repetitions as f64;

    // Add together squares of deviations from means all stats, then divide by number of repetitions to obtain variances.

    let mut variance_gnode_height = 0.0f64;
    let mut variance_gnode_count = 0.0f64;
    let mut variance_item_count = 0.0f64;
    let mut variance_item_slot_count = 0.0f64;
    let mut variance_space_amplification = 0.0f64;
    let mut variance_physical_height = 0.0f64;
    let mut variance_average_gnode_size = 0.0f64;
    let mut variance_height_amplification = 0.0f64;

    for (stats, phy_height) in results.iter() {
        variance_gnode_height += ((stats.gnode_height as f64) - gnode_height) * ((stats.gnode_height as f64) - gnode_height);
        variance_gnode_count += ((stats.gnode_count as f64) - gnode_count) * ((stats.gnode_count as f64) - gnode_count);
        variance_item_count += ((stats.item_count as f64) - item_count) * ((stats.item_count as f64) - item_count);
        variance_item_slot_count += ((stats.item_slot_count as f64) - item_slot_count) * ((stats.item_slot_count as f64) - item_slot_count);
        variance_space_amplification += (((stats.item_slot_count as f64) / (stats.item_count as f64)) - space_amplification) * (((stats.item_slot_count as f64) / (stats.item_count as f64)) - space_amplification);
        variance_physical_height += ((*phy_height as f64) - physical_height) * ((*phy_height as f64) - physical_height);
        variance_height_amplification += (((*phy_height as f64) / perfect_height) - height_amplification) * (((*phy_height as f64) / perfect_height) - height_amplification);
        variance_average_gnode_size += (((item_count as f64) / (gnode_count as f64)) - average_gnode_size) * (((item_count as f64) / (gnode_count as f64)) - average_gnode_size);
    }

    variance_gnode_height /= repetitions as f64;
    variance_gnode_count /= repetitions as f64;
    variance_item_count /= repetitions as f64;
    variance_item_slot_count /= repetitions as f64;
    variance_space_amplification /= repetitions as f64;
    variance_physical_height /= repetitions as f64;
    variance_height_amplification /= repetitions as f64;
    variance_average_gnode_size /= repetitions as f64;

    println!("n = {}; K = {}; {} repetitions", size, K, repetitions);
    println!("Legend: name <value> (<variance>)");
    println!("---------------------------------------");
    println!("Item count: {:#?} ({:#?})", item_count, variance_item_count);
    println!("Item slot count: {:#?} ({:#?})", item_slot_count, variance_item_slot_count);
    println!("Space amplification: {:#?} ({:#?})", space_amplification, variance_space_amplification);
    println!("G-node count: {:#?} ({:#?})", gnode_count, variance_gnode_count);
    println!("Average G-node size: {:#?} ({:#?})", average_gnode_size, variance_average_gnode_size);
    println!("G-node height: {:#?} ({:#?})", gnode_height, variance_gnode_height);
    println!("Actual height: {:#?} ({:#?})", physical_height, variance_physical_height);
    println!("Perfect height: {:#?}", perfect_height);
    println!("Height amplification: {:#?} ({:#?})", height_amplification, variance_height_amplification);
    println!("\n\n");
}

pub fn main() {
    for n in [10, 100, 1000, 10000, 10000] {
        repeated_experiment::<1, u64>(n, 1000);
        repeated_experiment::<3, u64>(n, 1000);
        repeated_experiment::<15, u64>(n, 1000);
        repeated_experiment::<63, u64>(n, 1000);
    }
    // let tree: GTree<NonemptyReverseKList<15, u64>> = random_klist_tree(1_000_000);
    // let (stats, ranks) = gtree_stats(&tree);
    // let phy_height = physical_height(&tree);
    
    // println!("{:#?}", stats);
    // println!("physical height: {:#?}", phy_height);
    // println!("rank distribution {:#?}", ranks);

    // if stats.item_count < 10 {
    //     println!("{:#?}", tree);
    // }
}