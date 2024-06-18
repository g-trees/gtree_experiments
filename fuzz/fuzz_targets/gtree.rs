#![no_main]
use libfuzzer_sys::fuzz_target;

use gtree_experiments::{*, klist::*};

fuzz_target!(|data: TreeCreation<u8>| {
    let gtree: GTree<NonemptyReverseKList<3, u8>> = create_tree(data.clone());
    // let gtree: GTree<ControlSet<u8>> = create_tree(data.clone());
    let ctrl = create_ctrl_tree(data);

    for i in 0..=255 {
        let has_gtree = has(&gtree, &i);
        let has_ctrl = ctrl.contains(&i);

        if has_gtree != has_ctrl {
            println!("\n\nDifferent search results, searching for {:?}.\n{:#?}\n{:#?}", i, gtree, ctrl);
        }

        assert_eq!(has_gtree, has_ctrl);
    }
});
