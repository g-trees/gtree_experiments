#![no_main]
use libfuzzer_sys::fuzz_target;

use gtree_experiments::{*, klist::*};

fuzz_target!(|data: SetCreationOperation<u8>| {
    let ctrl: Option<Set<ControlSet<u8>>> = create_set(data.clone());
    if let Some(ctrl) = ctrl {
        let klist: Set<NonemptyReverseKList<3, u8>> = create_set(data.clone()).unwrap();

        match (ctrl, klist) {
            (Set::Empty, Set::Empty) => {/* no-op, all good */}
            (Set::NonEmpty(ctrl), Set::NonEmpty(klist)) => {
                sets_assert_eq(&klist, &ctrl);
            }
            (ctrl, klist) => {
                println!("klist: {:?}", klist);
                println!("ctrl:  {:?}", ctrl);
                panic!("Nonequal klist and control.");
            }
        }        
    }
});
