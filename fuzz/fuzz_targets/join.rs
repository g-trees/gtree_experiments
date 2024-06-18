#![no_main]
use libfuzzer_sys::fuzz_target;

use std::collections::HashSet;

use gtree_experiments::{klist::*, *};

fuzz_target!(|data_: (&[u8], usize)| {
    let (data, split) = data_;
    if split == 0 {
        return;
    }

    let mut deduplicator = HashSet::new();
    for n in data {
        deduplicator.insert(*n);
    }

    let mut v: Vec<u8> = deduplicator.iter().map(|x| *x).collect();
    v.sort_by(|a, b| b.cmp(a));

    if split >= v.len() {
        return;
    }

    let v1 = v[0..split].to_vec();
    let v2 = v[split..].to_vec();

    let klist1: NonemptyReverseKList<3, u8> = NonemptyReverseKList::from_descending(&v1);
    let klist2: NonemptyReverseKList<3, u8> = NonemptyReverseKList::from_descending(&v2);

    // println!("\n\nlist 2: {:#?}\n", klist2);
    // println!("\n\nlist 1: {:#?}\n", klist1);

    let klist = NonemptyReverseKList::join(&klist2, &klist1);

    // let klist: NonemptyReverseKList<3, u8> = NonemptyReverseKList::from_descending(&v);
    let ctrl = ControlSet(v.iter().map(|x| (*x, GTree::Empty)).collect());

    sets_assert_eq(&klist, &ctrl);
});
