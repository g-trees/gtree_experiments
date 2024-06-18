#![no_main]
use libfuzzer_sys::fuzz_target;

use std::collections::HashSet;

use gtree_experiments::{klist::*, *};

fuzz_target!(|data_: (&[u8], u8)| {
    let (data, split) = data_;
    if data.len() == 0 {
        return;
    }

    let mut deduplicator = HashSet::new();
    for n in data {
        deduplicator.insert(*n);
    }

    let mut v: Vec<u8> = deduplicator.iter().map(|x| *x).collect();
    v.sort();

    let (mut v1, mut v2) = match v.binary_search(&split) {
        Ok(i) => {
            // println!("iok {}", i);
            (v[0..i].to_vec(), v[i+1..].to_vec())
        }
        Err(i) => {
            // println!("ierr {}", i);
            (v[0..i].to_vec(), v[i..].to_vec())
        }
    };
    
    v.sort_by(|a, b| b.cmp(a));
    v1.sort_by(|a, b| b.cmp(a));
    v2.sort_by(|a, b| b.cmp(a));

    let klist: NonemptyReverseKList<3, u8> = NonemptyReverseKList::from_descending(&v);

    let (klist1, _, klist2) = NonemptyReverseKList::split(&klist, &split);

    let ctrl1_ = ControlSet(v1.iter().map(|x| (*x, GTree::Empty)).collect());
    let ctrl1 = if ctrl1_.0.len() == 0 {Set::Empty} else {Set::NonEmpty(ctrl1_)};
    let ctrl2_ = ControlSet(v2.iter().map(|x| (*x, GTree::Empty)).collect());
    let ctrl2 = if ctrl2_.0.len() == 0 {Set::Empty} else {Set::NonEmpty(ctrl2_)};

    // println!("\n\nsplit: {:#?}", split);
    // println!("\nklist: {:#?}\n", klist);
    // println!("\n\nv: {:#?}", v);
    // println!("\n\nv1: {:#?}", v1);
    // println!("\n\nv2: {:#?}", v2);

    possibly_empty_sets_assert_eq(&klist1, &ctrl1);
    possibly_empty_sets_assert_eq(&klist2, &ctrl2);
});
