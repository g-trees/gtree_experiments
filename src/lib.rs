// This file implements G-trees.
#![feature(maybe_uninit_write_slice)]

pub mod klist;

use std::rc::Rc;
use std::fmt::Debug;

use arbitrary::{Arbitrary};

/*
Definitions for NonemptySet and GTrees.
*/
#[derive(Debug, Clone)]
pub enum Set<S> {
    NonEmpty(S),
    Empty,
}

impl<S: NonemptySet> Set<S> {
    fn insert_min(&self, new_min: (S::Item, GTree<S>)) -> S {
        match self {
            Set::Empty => return S::singleton(new_min),
            Set::NonEmpty(s) => return s.insert_min(new_min),
        }
    }
}

pub trait NonemptySet: Clone
where
    Self: Sized,
{
    type Item;

    fn singleton(item: (Self::Item, GTree<Self>)) -> Self;
    fn split(&self, key: &Self::Item) -> (Set<Self>, Option<GTree<Self>> /* left subtree of key (if key is in self, else None) */, Set<Self>);
    fn join(left: &Self, right: &Self) -> Self;
    fn remove_min(&self) -> ((Self::Item, GTree<Self>), Set<Self>);
    fn insert_min(&self, new_min: (Self::Item, GTree<Self>)) -> Self;
}

#[derive(Debug, Clone)]
pub struct GTreeNode<S: NonemptySet> {
    set: S,
    right: GTree<S>,
    rank: u8,
}

#[derive(Debug, Clone)]
pub enum GTree<S: NonemptySet> {
    NonEmpty(Rc<GTreeNode<S>>),
    Empty,
}

fn update_leftmost<S: NonemptySet>(node: &GTreeNode<S>, leftmost: GTree<S>) -> Rc<GTreeNode<S>> {
    let ((leftmost_item, _), other_pairs) = node.set.remove_min();

    return Rc::new(GTreeNode {
        set: other_pairs.insert_min((leftmost_item, leftmost)),
        right: node.right.clone(),
        rank: node.rank,
    });
}

fn update_right<S: NonemptySet>(node: &GTreeNode<S>, right: GTree<S>) -> Rc<GTreeNode<S>> {
    return Rc::new(GTreeNode {
        set: node.set.clone(),
        right: right,
        rank: node.rank,
    });
}

// A (non-empty) GTree has a root GTreeNode that consists of a rank, a right subtree, and a non-empty set of pairs of items and their left subtrees.
// Occasionally, we need to construct a nonempty GTree from a rank, a right subtree, and a *possibly empty* set of pairs of items and their left subtrees. In those cases, if the set is empty, the resulting GTree is simply the supplied right subtree.
fn lift<S: NonemptySet>(s: &Set<S>, right: GTree<S>, rank: u8) -> GTree<S> {
    match s {
        Set::Empty => return right,
        Set::NonEmpty(set) => return GTree::NonEmpty(Rc::new(GTreeNode {
            rank,
            set: set.clone(),
            right,
        })),
    };
}

pub fn unzip<S: NonemptySet>(
    t: &GTree<S>,
    key: &S::Item,
) -> (GTree<S>, GTree<S>) {
    match t {
        // Empty tree is trivial to unzip.
        GTree::Empty => return (GTree::Empty, GTree::Empty),

        GTree::NonEmpty(s) => match s.set.split(key) {
            // If the current node contain the split point, everything until the split point becomes the left return, with the left child of the split point turning into the right child of the left return. Everything after the split point becomes the right return, with the right child of the current node becoming the right child of the right return.
            (left_set, Some(left_subtree_of_key), right_set) => {
                return (
                    lift(&left_set, left_subtree_of_key, s.rank),
                    lift(&right_set, s.right.clone(), s.rank),
                );
            }

            (_, None, Set::Empty) => {
                // If the current node does not contain the split point, and all its items are less than the split point, then recursively split its right child (and replace it with its left recursive return).
                let (left, right) = unzip(&s.right, key);
                return (
                    GTree::NonEmpty(update_right(s, left)),
                    right,
                );                
            }

            (left_set, None, Set::NonEmpty(r)) => {
                // If the current node does not contain the split point, but it does contain items greater than the split point, we need to split in the leftmost child of those greater items.
                let ((_, r_leftmost_subtree), _) = r.remove_min();
                let (left, right) = unzip(&r_leftmost_subtree, key);
                return (
                    lift(&left_set, left.clone(), s.rank),
                    GTree::NonEmpty(update_leftmost(s, right)),
                );
            }
        },
    }
}

pub fn zip2<S: NonemptySet>(
    left: &GTree<S>,
    right: &GTree<S>,
) -> GTree<S> {
    match (left, right) {
        (GTree::Empty, _) => return right.clone(),
        (_, GTree::Empty) => return left.clone(),
        (GTree::NonEmpty(l), GTree::NonEmpty(r)) => {
            if l.rank < r.rank {
                // Zip l into the leftmost subtree of r.
                let ((_, r_leftmost_subtree), _) = r.set.remove_min();
                let zipped = zip2(left, &r_leftmost_subtree);
                return GTree::NonEmpty(update_leftmost(r, zipped));
            } else if l.rank > r.rank {
                // Zip r into the right subtree of l.
                let zipped = zip2(&l.right, right);
                return GTree::NonEmpty(update_right(l, zipped));
            } else {
                // Equal ranks. Join the two inner sets, with the right subtree of the left node being zipped into the leftmost subtree of the right node.
                let ((r_leftmost_item, r_leftmost_subtree), r_others) = r.set.remove_min();
                let zipped = zip2(&l.right, &r_leftmost_subtree);
                let right_set = r_others.insert_min((r_leftmost_item, zipped));

                return GTree::NonEmpty(Rc::new(GTreeNode {
                    rank: l.rank, // same as r.rank
                    set: NonemptySet::join(&l.set, &right_set),
                    right: r.right.clone(),
                }));
            }
        }
    }
}

pub fn zip3<S: NonemptySet>(
    left: &GTree<S>,
    item: S::Item,
    rank: u8,
    right: &GTree<S>,
) -> GTree<S> {
    let mid = GTree::NonEmpty(Rc::new(GTreeNode {
        rank,
        set: S::singleton((item, GTree::Empty)),
        right: GTree::Empty,
    }));
    return zip2(&zip2(&left, &mid), &right);
}

pub fn insert<S: NonemptySet>(
    t: &GTree<S>,
    item: S::Item,
    rank: u8,
) -> GTree<S> {
    let (left, right) = unzip(t, &item);
    return zip3(&left, item, rank, &right);
}

pub fn delete<S: NonemptySet>(
    t: &GTree<S>,
    item: S::Item,
) -> GTree<S> {
    let (left, right) = unzip(t, &item);
    return zip2(&left, &right);
}

/// Additional methods for NonemptySets, to allow for testing and statistics gathering.
pub trait NonemptySetMeta: NonemptySet + Debug
where
    Self: Sized,
{
    /// Return a reference to the maximal item in the set.
    fn get_max(&self) -> &Self::Item;
    /// Return a reference to the minimal item in the set.
    fn get_min(&self) -> &Self::Item;
    /// Get the number of items in the set.
    fn len(&self) -> usize;
    // Get an item by index, where index 0 denotes the least item.
    fn get_by_index(&self, index: usize) -> Option<&Self::Item>;
    // Create an instance from a non-empty slice of strictly descending items (use empty trees as the left subtrees).
    fn from_descending(items: &[Self::Item]) -> Self;
}

pub fn sets_assert_eq<I: Debug + Eq, S1: NonemptySetMeta<Item = I>, S2: NonemptySetMeta<Item = I>>(s1: &S1, s2: &S2) {
    if s1.len() != s2.len() {
        println!("{:?}\n\n{:?}", s1, s2);
        let len = s1.len();
        assert_eq!(len, s2.len(), "Comparing lengths of the two sets.");
    }

    for i in 0..s1.len() {
        if s1.get_by_index(i) != s2.get_by_index(i) {
            println!("\n\n{:#?}\n\n{:?}\n\n", s1, s2);
            assert_eq!(s1.get_by_index(i), s2.get_by_index(i));
        }
    }
}

/*
Implementation of NonemptySet for a sorted (in descending order) Vec for testing purposes.
*/
#[derive(Debug, Clone)]
pub struct ControlSet<I: Clone + Ord>(pub Vec<(I, GTree<Self>)>);

impl<I: Clone + Ord> NonemptySet for ControlSet<I> {
    type Item = I;

    fn singleton(item: (Self::Item, GTree<Self>)) -> Self {
        return ControlSet(vec![item]);
    }

    fn insert_min(&self, new_min: (Self::Item, GTree<Self>)) -> Self {
        let mut ret = self.clone();
        ret.0.push(new_min);
        return ret;
    }

    fn remove_min(&self) -> ((Self::Item, GTree<Self>), Set<Self>) {
        let mut ret = self.clone();
        let popped = ret.0.pop().unwrap();
        return (popped, if ret.0.len() == 0 { Set::Empty } else { Set::NonEmpty(ret) });
    }

    fn split(&self, key: &Self::Item) -> (Set<Self>, Option<GTree<Self>> /* left subtree of key (if key is in self, else None) */, Set<Self>) {
        match self.0.binary_search_by(|(my_item, _)| {
            return key.cmp(my_item);
        }) {
            Ok(i) => {
                let right = self.0[0..i].to_vec();
                let left = self.0[i+1..].to_vec();
                return (
                    if left.len() == 0 { Set::Empty } else { Set::NonEmpty(ControlSet(left)) },
                    Some(self.0[i].1.clone()),
                    if right.len() == 0 { Set::Empty } else { Set::NonEmpty(ControlSet(right)) },
                );
            }
            Err(i) => {
                let right = self.0[0..i].to_vec();
                let left = self.0[i..].to_vec();
                return (
                    if left.len() == 0 { Set::Empty } else { Set::NonEmpty(ControlSet(left)) },
                    None,
                    if right.len() == 0 { Set::Empty } else { Set::NonEmpty(ControlSet(right)) },
                );
            }
        }
    }

    fn join(left: &Self, right: &Self) -> Self {
        let mut new = right.clone();
        new.0.extend_from_slice(&left.0[..]);
        return new;
    }
}

impl<I: Clone + Ord + Debug> NonemptySetMeta for ControlSet<I> {
    /// Return a reference to the maximal item in the set.
    fn get_max(&self) -> &Self::Item {
        return &self.0[0].0;
    }
    
    /// Return a reference to the minimal item in the set.
    fn get_min(&self) -> &Self::Item {
        return &self.0[self.0.len() - 1].0;
    }

    fn len(&self) -> usize {
        return self.0.len();
    }

    fn get_by_index(&self, index: usize) -> Option<&Self::Item> {
        return self.0.get(self.0.len() - (index + 1)).map(|(item, _)| item);
    }

    fn from_descending(items: &[Self::Item]) -> Self {
        let mut ret = Self::singleton((items[0].clone(), GTree::Empty));

        if items.len() == 1 {
            return ret;
        }

        for i in 1..items.len() {
            ret = ret.insert_min((items[i].clone(), GTree::Empty))
        }

        return ret;
    }
}

// Operations for constructing random sets. The subtrees in those sets are always empty.
#[derive(Debug, Arbitrary, Clone)]
pub enum SetCreationOperation<Item> {
    Singleton(Item),
    InsertMin(Box<Self>, Item),
    RemoveMin(Box<Self>),
    // Split and choose the left return value.
    // SplitLeft(Box<Self>, Item),
    // // Split and choose the right return value.
    // SplitRight(Box<Self>, Item),
    // Join(Box<Self>, Box<Self>),
}

// Try to create a set. Return None if a creation operation is invalid (ading a non-minimal item or joining non-disjoint ordered sets).
pub fn create_set<Item: Clone + Ord, S: NonemptySetMeta<Item = Item>>(creation: SetCreationOperation<Item>) -> Option<Set<S>> {
    match creation {
        SetCreationOperation::Singleton(item) => {
            return Some(Set::NonEmpty(S::singleton((item, GTree::Empty))));
        }
        SetCreationOperation::InsertMin(creation_rec, item) => {
            match create_set::<_, S>(*creation_rec) {
                None => None,
                Some(set_rec) => {
                    match set_rec {
                        Set::Empty => return Some(Set::NonEmpty(S::singleton((item, GTree::Empty)))),
                        Set::NonEmpty(neset_rec) => {
                            if neset_rec.get_min() <= &item {
                                return None;
                            } else {
                                return Some(Set::NonEmpty(neset_rec.insert_min((item, GTree::Empty))));
                            }
                        }
                    }
                }
            }
        }
        SetCreationOperation::RemoveMin(creation_rec) => {
            match create_set::<_, S>(*creation_rec) {
                None => None,
                Some(set_rec) => {
                    match set_rec {
                        Set::Empty => return Some(Set::Empty),
                        Set::NonEmpty(neset_rec) => return Some(neset_rec.remove_min().1),
                    }
                }
            }
        }
        // SetCreationOperation::SplitLeft(creation_rec, item) => {
        //     match create_set::<_, S>(*creation_rec) {
        //         None => None,
        //         Some(set_rec) => {
        //             match set_rec {
        //                 Set::Empty => return Some(Set::Empty),
        //                 Set::NonEmpty(neset_rec) => {
        //                     return Some(neset_rec.split(&item).0);
        //                 }
        //             }
        //         }
        //     }
        // }
        // SetCreationOperation::SplitRight(creation_rec, item) => {
        //     match create_set::<_, S>(*creation_rec) {
        //         None => None,
        //         Some(set_rec) => {
        //             match set_rec {
        //                 Set::Empty => return Some(Set::Empty),
        //                 Set::NonEmpty(neset_rec) => {
        //                     return Some(neset_rec.split(&item).2);
        //                 }
        //             }
        //         }
        //     }
        // }
        // SetCreationOperation::Join(left_creation_rec, right_creation_rec) => {
        //     match (create_set::<_, S>(*left_creation_rec), create_set::<_, S>(*right_creation_rec)) {
        //         (Some(left_set_rec), Some(right_set_rec)) => {
        //             match (left_set_rec, right_set_rec) {
        //                 (Set::Empty, Set::Empty) => return Some(Set::Empty),
        //                 (Set::NonEmpty(left), Set::Empty) => return Some(Set::NonEmpty(left)),
        //                 (Set::Empty, Set::NonEmpty(right)) => return Some(Set::NonEmpty(right)),
        //                 (Set::NonEmpty(left), Set::NonEmpty(right)) => {
        //                     if left.get_max() < right.get_min() {
        //                         return Some(Set::NonEmpty(NonemptySet::join(&left, &right)));
        //                     } else {
        //                         return None;
        //                     }
        //                 }
        //             }
        //         }
        //         _=> return None,
        //     }
        // }
    }
}




// pub enum TreeCreation<Item> {
//     Empty,
//     Insert(Box<Self>, Item, u8),
//     Remove(Box<Self>, Item),
// }


// // Create a tree according to a TreeDescription value, also report its least and greatest item (if it is non-empty).
// pub fn create_tree<Item: Clone + Ord, S: NonemptySet<Item = Item>>(creation: TreeCreation<Item>) -> (GTree<S>, Option<(Item, Item)>) {
//     match creation {
//         TreeCreation::Empty => return (GTree::Empty, None),
//         TreeCreation::Insert(creation_rec, item, rank) => {
//             let (tree_rec, extrema_rec) = create_tree(*creation_rec);
//             let new_tree = insert(&tree_rec, item.clone(), rank);

//             match extrema_rec {
//                 None => return (new_tree, Some((item.clone(), item.clone()))),
//                 Some((old_min, old_max)) => return (
//                     new_tree,
//                     Some((std::cmp::min(old_min, item.clone()), std::cmp::max(old_max, item.clone()))),
//                 ),
//             }
//         }
//         TreeCreation::Remove(creation_rec, item) => {
//             let (tree_rec, extrema_rec) = create_tree(*creation_rec);
//             let new_tree = delete(&tree_rec, item.clone());

//             match extrema_rec {
//                 None => return (new_tree, Some((item.clone(), item.clone()))),
//                 Some((old_min, old_max)) => return (
//                     new_tree,
//                     Some((std::cmp::min(old_min, item.clone()), std::cmp::max(old_max, item.clone()))),
//                 ),
//             }
//         }
//     }
// }