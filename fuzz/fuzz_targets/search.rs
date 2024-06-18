#![no_main]
use libfuzzer_sys::fuzz_target;

use std::collections::HashSet;

use gtree_experiments::{klist::*, *};

fuzz_target!(|data_: (&[u8], u8)| {
    let (data, key) = data_;
    if data.len() == 0 {
        return;
    }

    let mut deduplicator = HashSet::new();
    for n in data {
        deduplicator.insert(*n);
    }

    let mut v: Vec<u8> = deduplicator.iter().map(|x| *x).collect();
    v.sort_by(|a, b| b.cmp(a));

    let klist: NonemptyReverseKList<3, u8> = NonemptyReverseKList::from_descending(&v);

    let ctrl = ControlSet(v.iter().map(|x| (*x, GTree::Empty)).collect());

    let ctrl_found = ctrl.search(&key).map(|(item, _subtree)| item.clone());
    let found = klist.search(&key).map(|(item, _subtree)| item.clone());

    assert_eq!(found, ctrl_found);

    // println!("\n\nsplit: {:#?}", split);
    // println!("\nklist: {:#?}\n", klist);
    // println!("\n\nv: {:#?}", v);
    // println!("\n\nv1: {:#?}", v1);
    // println!("\n\nv2: {:#?}", v2);
});
