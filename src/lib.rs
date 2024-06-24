// This file implements G-trees.
#![feature(maybe_uninit_write_slice)]

pub mod klist;

use std::collections::BTreeMap;
use std::{collections::BTreeSet, rc::Rc};
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
    /// Return the item-left_subtree pair witht the least item that is greater than or equal to `key`. Return None if no such pair exists.
    fn search(&self, key: &Self::Item) -> Option<(Self::Item, GTree<Self>)>;
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

pub fn unzip<S: NonemptySet + Debug>(
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
                // println!("just_split {:#?}\n{:#?}", left_set, r);
                // If the current node does not contain the split point, but it does contain items greater than the split point, we need to split in the leftmost child of those greater items.
                let ((r_leftmost_item, r_leftmost_subtree), r_remaining) = r.remove_min();
                let (left, right) = unzip(&r_leftmost_subtree, key);
                // println!("recursive unzip {:#?}\n{:#?}", left, right);
                // println!("split returning\n{:#?}\n{:#?}", lift(&left_set, left.clone(), s.rank), GTree::NonEmpty(update_leftmost(s, right.clone())));
                // return (
                //     lift(&left_set, left.clone(), s.rank),
                //     GTree::NonEmpty(update_leftmost(s, right)),
                // );
                let right_return = GTree::NonEmpty(Rc::new(GTreeNode {
                    rank: s.rank,
                    set: r_remaining.insert_min((r_leftmost_item, right)),
                    right: s.right.clone(),
                }));
                // let right_return = GTree::NonEmpty(update_leftmost(&GTreeNode {
                //     set: r,
                //     rank: s.rank,
                //     right: s.right.clone(),
                // }, right));
                // println!("split returning\n{:#?}\n{:#?}", lift(&left_set, left.clone(), s.rank), right_return);
                return (
                    lift(&left_set, left.clone(), s.rank),
                    right_return,
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

pub fn insert<S: NonemptySet + Debug>(
    t: &GTree<S>,
    item: S::Item,
    rank: u8,
) -> GTree<S> {
    // println!("inserting into {:#?}\n", t);
    let (left, right) = unzip(t, &item);
    // println!("a unzipped {:#?}\n{:#?}", left, right);
    let zipped = zip3(&left, item, rank, &right);
    // println!("zipped {:#?}\n\n\n\n", zipped);
    return zipped;
}

pub fn delete<S: NonemptySet + Debug>(
    t: &GTree<S>,
    item: &S::Item,
) -> GTree<S> {
    let (left, right) = unzip(t, item);
    return zip2(&left, &right);
}

pub fn has<S: NonemptySet>(
    t: &GTree<S>,
    key: &S::Item,
) -> bool where S::Item: Ord {
    match t {
        GTree::Empty => return false,
        GTree::NonEmpty(node) => {
            match node.set.search(key) {
                None => return has(&node.right, key),
                Some(yay) => {
                    if &yay.0 == key {
                        return true;
                    } else {
                        return has(&yay.1, key);
                    }
                }
            }
        }
    }
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
    // Get an item and its left subtree by index, where index 0 denotes the least item.
    fn get_pair_by_index(&self, index: usize) -> Option<&(Self::Item, GTree<Self>)>;
    // Get an item by index, where index 0 denotes the least item.
    fn get_by_index(&self, index: usize) -> Option<&Self::Item> {
        return self.get_pair_by_index(index).map(|(item, _)| item);
    }
    // Create an instance from a non-empty slice of strictly descending items (use empty trees as the left subtrees).
    fn from_descending(items: &[Self::Item]) -> Self;
    // Total number of items this could store without allocating more memory. Used to compute space amplification.
    fn item_slot_count(&self) -> usize;
}

// Return a vec of item-left_subtree pairs in descending order.
fn pairs_ascending<S: NonemptySetMeta>(s: &S) -> Vec<&(S::Item, GTree<S>)> where S::Item: Ord {
    let mut ret  = vec![];

    for i in 0..s.len() {
        let pair = s.get_pair_by_index(i).unwrap();
        ret.push(pair);
    }

    ret.sort_by(|(item_a, _), (item_b, _)| item_a.cmp(item_b));
    return ret;
}

#[derive(Clone, Debug)]
pub struct Stats<Item> {
    pub gnode_height: usize, // empty tree has height 0
    pub gnode_count: usize,
    pub item_count: usize,
    pub item_slot_count: usize,
    pub rank: i16,
    pub is_heap: bool,
    pub least_item: Option<Item>,
    pub greatest_item: Option<Item>,
    pub is_search_tree: bool,
}

pub fn gtree_stats<S: NonemptySetMeta>(
    t: &GTree<S>,
) -> (Stats<S::Item>, BTreeMap<u8, usize> /* rank distribution */) where S::Item: Clone + Ord + Debug {
    let mut ranks = BTreeMap::new();
    let stats = gtree_stats_(t, &mut ranks);

    return (stats, ranks);
}

fn gtree_stats_<S: NonemptySetMeta>(
    t: &GTree<S>,
    rank_distribution: &mut BTreeMap<u8, usize>,
) -> Stats<S::Item> where S::Item: Clone + Ord + Debug {
    match t {
        GTree::Empty => {
            return Stats {
                gnode_height: 0,
                gnode_count: 0,
                item_count: 0,
                item_slot_count: 0,
                rank: -1,
                is_heap: true,
                least_item: None,
                greatest_item: None,
                is_search_tree: true,
            }
        }
        GTree::NonEmpty(node) => {
            // Get stats for all children.
            let pairs = pairs_ascending(&node.set);

            match rank_distribution.get(&node.rank) {
                None => rank_distribution.insert(node.rank, pairs.len()),
                Some(prev) => rank_distribution.insert(node.rank, prev + pairs.len()),
            };

            let pair_stats: Vec<_> = pairs.into_iter()
                .map(|(item, subtree)| (item, gtree_stats_(subtree, rank_distribution))).collect();
            let right_stats = gtree_stats_(&node.right, rank_distribution);

            let mut stats = right_stats.clone();

            /*
             * Simple gnode properties.
             */

            stats.gnode_height = 1 + std::cmp::max(right_stats.gnode_height, pair_stats.iter().map(|(_, stats)| stats.gnode_height).max().unwrap());

            // stats.gnode_count starts out as right_stats.gnode_count
            stats.gnode_count += 1; //counting itself
            for (_, left_subtree_stats) in pair_stats.iter() {
                stats.gnode_count += left_subtree_stats.gnode_count;
            }

            // stats.item_count starts out as right_stats.item_count
            for (_, left_subtree_stats) in pair_stats.iter() {
                stats.item_count += 1 + left_subtree_stats.item_count;
            }

            /*
             * Item slot count.
             */
            // stats.item_slot_count starts out as right_stats.item_slot_count
            for (_, left_subtree_stats) in pair_stats.iter() {
                stats.item_slot_count += left_subtree_stats.item_slot_count;
            }
            stats.item_slot_count += node.set.item_slot_count();

            /*
             * Ranks and heap property.
             */
            stats.rank = node.rank.into();

            for (i, (_, left_subtree_stats)) in pair_stats.iter().enumerate() {
                if left_subtree_stats.rank >= node.rank.into() {
                    stats.is_heap = false;
                    println!("\n\n heap property: subtree {} rank too great\n{:#?}\n\n", i, t);
                    println!("\npair_stats {:#?}\n", pair_stats);
                }

                if !left_subtree_stats.is_heap {
                    stats.is_heap = false;
                }
            }

            if right_stats.rank > node.rank.into() {
                stats.is_heap = false;
                println!("\n\n heap property: right subtree rank too great\n{:#?}\n\n", t);
                println!("\right_stats {:#?}\n", right_stats);
            } else if right_stats.rank == node.rank.into() {
                if node.set.item_slot_count() > node.set.len() {
                    println!("\n\n heap property: right subtree equal but free slots\n{:#?}\n\n", t);
                    println!("\right_stats {:#?}\n", right_stats);
                    stats.is_heap = false;
                }
            }

            if !right_stats.is_heap {
                stats.is_heap = false;
            }

            /*
             * Check search tree property.
             */
            // Right descendents are greater than the greatest item in the node.
            if let Some(ref least) = right_stats.least_item {
                if least <= pair_stats[pair_stats.len() - 1].0 {
                    stats.is_search_tree = false;
                    println!("\n\n search tree property: right too great\n{:#?}\n\n", t);
                }
            }
            for (i, (item, left_subtree_stats)) in pair_stats.iter().enumerate() {
                if let Some(ref least) = left_subtree_stats.least_item {
                    // All left descendents are greater than their parent's left sibling
                    if i > 0 && least <= pair_stats[i - 1].0 {
                        stats.is_search_tree = false;
                        println!("\n\n search tree property: left {} too small\n{:#?}\n\n", i, t);
                        println!("\npair_stats {:#?}\n", pair_stats);
                    }
                }

                if let Some(ref greatest) = left_subtree_stats.greatest_item {
                    // All left descendents are less than their parent
                    if greatest >= item {
                        stats.is_search_tree = false;
                        println!("\n\n search tree property: left {} too great\n{:#?}\n\n", i, t);
                    }
                }
            }

            // Set least and greatest item of self.
            let least_pair = &pair_stats[0];
            match least_pair.1.least_item {
                Some(ref least) => stats.least_item = Some(least.clone()),
                None => stats.least_item = Some(least_pair.0.clone()),
            }
            match right_stats.greatest_item {
                Some(greatest) => stats.greatest_item = Some(greatest.clone()),
                None => stats.greatest_item = Some(pair_stats[pair_stats.len() - 1].0.clone()),
            }

            return stats;
        }
    }
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

pub fn possibly_empty_sets_assert_eq<I: Debug + Eq, S1: NonemptySetMeta<Item = I>, S2: NonemptySetMeta<Item = I>>(s1: &Set<S1>, s2: &Set<S2>) {
    match (s1, s2) {
        (Set::Empty, Set::Empty) => {
            // no-op
        }
        (Set::NonEmpty(s1), Set::NonEmpty(s2)) => return sets_assert_eq(s1, s2),
        _ => {
            panic!("\n\nGot non-equal sets:\n\n{:#?}\n\n{:?}\n\n", s1, s2);
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
    
    fn search(&self, key: &Self::Item) -> Option<(Self::Item, GTree<Self>)> {
        match self.0.binary_search_by(|x| key.cmp(&x.0)) {
            Ok(i) => {
                return Some(self.0[i].clone());
            }
            Err(i) => {
                if i == 0 {
                    return None;
                } else {
                    return Some(self.0[i - 1].clone());
                }
            }
        }
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

    fn get_pair_by_index(&self, index: usize) -> Option<&(Self::Item, GTree<Self>)> {
        return self.0.get(self.0.len() - (index + 1));
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

    fn item_slot_count(&self) -> usize {
        return self.len();
    }
}

// Operations for constructing simple random sets. The subtrees in those sets are always empty.
#[derive(Debug, Arbitrary, Clone)]
pub enum SetCreationOperation<Item> {
    Singleton(Item),
    InsertMin(Box<Self>, Item),
    RemoveMin(Box<Self>),
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
    }
}



#[derive(Debug, Arbitrary, Clone)]
pub enum TreeCreation<Item> {
    Empty,
    Insert(Box<Self>, Item, u8),
    Remove(Box<Self>, Item),
}

// Create a tree according to a TreeDescription value.
pub fn create_tree<Item: Clone + Ord, S: NonemptySet<Item = Item> + Debug>(creation: TreeCreation<Item>) -> GTree<S> {
    match creation {
        TreeCreation::Empty => return GTree::Empty,
        TreeCreation::Insert(creation_rec, item, rank) => {
            let tree_rec = create_tree(*creation_rec);
            let new_tree = insert(&tree_rec, item.clone(), rank);
            return new_tree;
        }
        TreeCreation::Remove(creation_rec, item) => {
            let tree_rec = create_tree(*creation_rec);
            let new_tree = delete(&tree_rec, &item);
            return new_tree;
        }
    }
}

pub fn create_ctrl_tree<Item: Clone + Ord>(creation: TreeCreation<Item>) -> BTreeSet<Item> {
    match creation {
        TreeCreation::Empty => return BTreeSet::new(),
        TreeCreation::Insert(creation_rec, item, _rank) => {
            let mut tree_rec = create_ctrl_tree(*creation_rec);
            tree_rec.insert(item);
            return tree_rec;
        }
        TreeCreation::Remove(creation_rec, item) => {
            let mut tree_rec = create_ctrl_tree(*creation_rec);
            tree_rec.remove(&item);
            return tree_rec;
        }
    }
}