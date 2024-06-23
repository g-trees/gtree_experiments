use std::{cmp::*, rc::Rc, fmt::Debug};

use crate::{Set, GTree, NonemptySet, NonemptySetMeta};

/// A k-list, except for a few optimization details:
///
/// - the list is known to be non-empty, so it always contains at least one vertex, and
/// - the list stores its items in reverse order, to enable `insert_min` and `remove_min` in O(1) rather than O(n) time.
#[derive(Debug, Clone)]
pub struct NonemptyReverseKList<const K: usize, I: Clone + Ord + Debug> {
    data: [Option<(I, GTree<Self>)>; K],
    next: Option<Rc<Self>>,
}

impl<const K: usize, I: Clone + Ord + Debug> NonemptyReverseKList<K, I> {
    // Internal helper function: remove the `n` greatest items from a list, with 1 <= n <= K.
    // Returns first the (up to n) items that were removed, then the valid remaining list (or None if it would be empty).
    fn remove_n_max(&self, n: usize) -> ([Option<(I, GTree<Self>)>; K], Option<Self>) {
        if n == 0 || n > K {
            unreachable!("Violated internal invariant!");
        } else if n == K {
            // Simply pop off the whole first vertex.
            return (
                self.data.clone(),
                match self.next {
                    None => None,
                    Some(ref next) => Some((**next).clone()),
                },
            );
        } else {
            // We need to remove the first n items.
            // println!("n {}", n);

            // First, we pop off the first n items
            let removed: [_; K] = std::array::from_fn(|i| {
                if i < n {
                    return self.data[i].clone();
                } else {
                    return None;
                }
            });

            // println!("\nremoved: {:#?}\n", removed);

            // Next, we move the remaining items of the current vertex to the front.
            let mut new_data: [_; K] = std::array::from_fn(|i| {
                if i + n < K {
                    return self.data[i + n].clone();
                } else {
                    return None;
                }
            });

            // println!("\nnew_data: {:#?}\n", new_data);

            match self.next {
                None => {
                    // If we are the last vertex, we are done.
                    return (
                        removed,
                        match new_data[0] {
                            None => None,
                            Some(_) => Some(NonemptyReverseKList {
                                data: new_data,
                                next: None,
                            }),
                        },
                    );
                }
                Some(ref next) => {
                    // Recurse on the remaining vertices.
                    let (removed_rec, remaining_rec) = next.remove_n_max(n);

                    // println!("\nnew_data before: {:#?}\n", new_data);
                    // println!("\nremoved_rec: {:#?}\n", removed_rec);

                    // Copy the recursively removed items into the new data for this vertex.
                    for i in 0..n {
                        // println!("i {} n {} K {}", i, n, K);
                        new_data[(K - n) + i] = removed_rec[i].clone();
                    }

                    // println!("\nnew_data again: {:#?}\n", new_data);

                    return (
                        removed,
                        match new_data[0] {
                            None => None,
                            Some(_) => Some(NonemptyReverseKList {
                                data: new_data,
                                next: remaining_rec.map(Rc::new),
                            }),
                        },
                    );
                }
            }
        }
    }

    // Internal helper function: get an item by index, where index 0 denotes the *greatest* item.
    fn get_pair_by_inverted_index(&self, index: usize) -> Option<&(I, GTree<Self>)>{
        if index < K {
            return self.data[index].as_ref();
        } else {
            match self.next {
                Some(ref next) => return next.get_pair_by_inverted_index(index - K),
                None => return None,
            }
        }
    }
}

impl<const K: usize, I: Clone + Ord + Debug> NonemptySet for NonemptyReverseKList<K, I> {
    type Item = I;

    fn singleton(item: (Self::Item, GTree<Self>)) -> Self {
        let mut data = std::array::from_fn(|_| None);
        data[0] = Some(item);

        return NonemptyReverseKList {
            data,
            next: None,
        }
    }

    fn insert_min(&self, new_min: (Self::Item, GTree<Self>)) -> Self {
        match self.next {
            // If self is not the final vertex, recurse.
            Some(ref next) => {
                let new_next = next.insert_min(new_min);
                let mut new_self = self.clone();
                new_self.next = Some(Rc::new(new_next));
                return new_self;
            }
            None => {
                // self is the final vertex, try to insert in the first free slot.
                for i in 0..K {
                    if let None = self.data[i] {
                        // Found a free slot; insert and return.
                        let mut new_data = self.data.clone();
                        new_data[i] = Some(new_min);

                        return NonemptyReverseKList {
                            data: new_data,
                            next: None,
                        }
                    }
                }

                // Found no free slot, append a new vertex.
                let new_vertex = Rc::new(NonemptyReverseKList::singleton(new_min));
                return NonemptyReverseKList {
                    data: self.data.clone(),
                    next: Some(new_vertex),
                }
            }
        }
    }

    fn remove_min(&self) -> ((Self::Item, GTree<Self>), Set<Self>) {
        match self.next {
            // If self is not the final vertex, recurse.
            Some(ref next) => {
                let (min, new_next) = next.remove_min();
                let mut new_self = self.clone();
                new_self.next = match new_next {
                    Set::Empty => None,
                    Set::NonEmpty(new_next) => Some(Rc::new(new_next)),
                };
                return (min, Set::NonEmpty(new_self));
            }
            None => {
                // self is the final vertex, remove from the last occupied slot.
                for i in (0..K).rev() {
                    if let Some(ref min) = self.data[i] {
                        // Found the last occupied slot (there is always at least one); remove and return.
                        if i == 0 {
                            // Vertex would become empty, so we "delete" it by returning the empty set.
                            return (min.clone(), Set::Empty);
                        } else {
                            // More items remain, so we create a copy of this vertex that does not conain the last item.
                            let mut new_data = self.data.clone();
                            new_data[i] = None;

                            return (min.clone(), Set::NonEmpty(NonemptyReverseKList {
                                data: new_data,
                                next: None,
                            }))
                        }
                    }
                }

                // There is always at least one occupied slot.
                unreachable!()
            }
        }
    }

    fn split(&self, key: &Self::Item) -> (Set<Self>, Option<GTree<Self>> /* left subtree of key (if key is in self, else None) */, Set<Self>) {
        // Try to find key in self.
        // Because we store items in reverse, we use a comparison function that compares in reverse as well.
        match self.data.binary_search_by(|opt| {
            match opt {
                // The `None`s are toward the end of the array, so `None`
                // must compare as greater than any item.
                None => return Ordering::Greater,
                // We compare the key with the stored item instead of the
                // other way round, to account for the reverse order.
                Some((my_item, _)) => return key.cmp(my_item),
            }
        }) {
            Ok(i) => {
                // We contain the key at index i.
                // println!("Ok i: {:?}", i);

                // First, we compute the right return.
                let right = if i == 0 {
                    // The key is the very first item, so the right return is the empty set.
                    Set::Empty
                } else {
                    // The right return contains all the items up to but excluding index i.
                    let right_data = std::array::from_fn(|j| {
                        if j < i {
                            return self.data[j].clone();
                        } else {
                            return None;
                        }
                    });
                    Set::NonEmpty(NonemptyReverseKList {
                        data: right_data, // safe to do this, i > 0, so right_data is not empty
                        next: None,
                    })
                };
                // println!("right {:#?}", right);

                // println!("a {:?}", self);
                // We obtain the left return by removing our first `i + 1` items.
                let left = self.remove_n_max(i + 1).1;
                // let (_, left) = if i == 0 { (None) } else { self.remove_n_max(i) };

                return (
                    match left {
                        None => Set::Empty,
                        Some(l) => Set::NonEmpty(l),
                    },
                    Some(self.data[i].as_ref().unwrap(/* binary search returned i*/).1.clone()),
                    right,
                );
            }
            Err(i) => {
                // We do not contain the key.
                // println!("Err i: {:?}", i);

                if i == 0 {
                    // We (and the whole list) contain only items less than the key.
                    return (Set::NonEmpty(self.clone()), None, Set::Empty);
                } else if i == K {
                    // We contain no item less than the key.

                    match self.next {
                        None => {
                            // All items are greater than the key.
                            return (Set::Empty, None, Set::NonEmpty(self.clone()));
                        }
                        Some(ref next) => {
                            // Recurse and append the right recursive return to ourselves.
                            let (left_rec, mid_rec, right_rec) = next.split(key);
                            // println!("recursive:\n{:#?}\n{:?}\n{:#?}", left_rec, mid_rec, right_rec);

                            match right_rec {
                                Set::Empty => {
                                    let mut cloned = self.clone();
                                    cloned.next = None;
                                    return (
                                        left_rec,
                                        mid_rec,
                                        Set::NonEmpty(cloned),
                                    );
                                }
                                Set::NonEmpty(right_rec) => {
                                    let mut cloned = self.clone();
                                    cloned.next = Some(Rc::new(right_rec));
                                    return (
                                        left_rec,
                                        mid_rec,
                                        Set::NonEmpty(cloned),
                                    );
                                }
                            }
                        }
                    }
                } else {
                    // i is less than K, so we contain i items greater than the key.

                    // First, we compute the right return.
                    // The right return contains all the items up to but excluding index i.
                    let right_data = std::array::from_fn(|j| {
                        if j < i {
                            return self.data[j].clone();
                        } else {
                            return None;
                        }
                    });
                    // println!("c {:?}", right_data);
                    let right = Set::NonEmpty(NonemptyReverseKList {
                        data: right_data,
                        next: None,
                    });

                    // println!("b {:?}", self);
                    // We obtain the left return by removing our first `i` items.
                    let (_, left) = self.remove_n_max(i);
                    // println!("d {:?}", left);
                    // println!("e {:?}", right);

                    return (
                        match left {
                            None => Set::Empty,
                            Some(l) => Set::NonEmpty(l),
                        },
                        None,
                        right,
                    );
                }
            }
        }
    }

    fn join(left: &Self, right: &Self) -> Self {
        // We need to *prepend* right to left, because we store things in reverse order.
        match right.next {
            Some(ref right_next) => {
                // Recurse and use the return value as the next vertex for the first vertex of `right`.
                return NonemptyReverseKList {
                    data: right.data.clone(),
                    next: Some(Rc::new(Self::join(left, right_next))),
                };
            }
            None => {
                // Actually need to do some work now.

                // How many items does `right` store?
                let mut right_count = 0;
                for i in 0..K {
                    match right.data[i] {
                        Some(_) => right_count += 1,
                        None => break,
                    }
                }

                // println!("right count {}", right_count);

                if right_count == K {
                    // Right is full, so we can simply set right.next to left.
                    return NonemptyReverseKList {
                        data: right.data.clone(),
                        next: Some(Rc::new(left.clone())),
                    };
                } else {
                    // Right has K - right_count free slots, so move that many items from left into right, and then concatenate.
                    let to_move = K - (right_count);
                    let (left_removed, left_remaining) = left.remove_n_max(to_move);
                    // println!("to_move {}, left_removed {:?}", to_move, left_removed);
                    let new_data: [_; K] = std::array::from_fn(|i| {
                        if i < right_count {
                            return right.data[i].clone();
                        } else {
                            return left_removed[i - right_count].clone();
                        }
                    });

                    // println!("new_data {:?}", new_data);
                    // println!("left_remaining {:?}", left_remaining);
                    
                    return NonemptyReverseKList {
                        data: new_data,
                        next: left_remaining.map(|l| Rc::new(l)),
                    };
                }
            }
        }
    }
    
    fn search(&self, key: &Self::Item) -> Option<(Self::Item, GTree<Self>)> {
        match self.data.binary_search_by(|opt| {
            match opt {
                // The `None`s are toward the end of the array, so `None`
                // must compare as greater than any item.
                None => return Ordering::Greater,
                // We compare the key with the stored item instead of the
                // other way round, to account for the reverse order.
                Some((my_item, _)) => return key.cmp(my_item),
            }
        }) {
            Ok(i) => {
                return self.data[i].clone();
            }
            Err(i) => {
                if i == 0 {
                    return None;
                } else if i == K {
                    match self.next {
                        None => return self.data[i - 1].clone(),
                        Some(ref next) => {
                            match next.search(key) {
                                None => return self.data[K - 1].clone(),
                                Some(yay) => return Some(yay),
                            }
                        }
                    }
                } else {
                    return self.data[i - 1].clone();
                }
            }
        }
    }
}

impl<const K: usize, I: Clone + Ord + Debug> NonemptySetMeta for NonemptyReverseKList<K, I> {
    /// Return a reference to the maximal item in the set.
    fn get_max(&self) -> &Self::Item {
        match self.data[0] {
            None => unreachable!("List is never empty."),
            Some((ref item, ref _subtree)) => return item,
        }
    }

    /// Return a reference to the minimal item in the set.
    fn get_min(&self) -> &Self::Item {
        match self.next {
            Some(ref next) => {
                return next.get_min();
            }
            None => {
                for i in (0..K).rev() {
                    if let Some((ref item, _)) = self.data[i] {
                        return &item;
                    }
                }
                unreachable!("self.data contains at least one item.");
            }
        }
    }

    fn len(&self) -> usize {
        match self.next {
            Some(ref next) => {
                return K + next.len();
            }
            None => {
                for i in (0..K).rev() {
                    if let Some(_) = self.data[i] {
                        return i + 1;
                    }
                }
                println!("{:?}", self);
                unreachable!("self.data contains at least one item.");
            }
        }
    }

    fn item_slot_count(&self) -> usize {
        match self.next {
            Some(ref next) => {
                return K + next.item_slot_count();
            }
            None => {
                return K;
            }
        }
    }

    fn get_pair_by_index(&self, index: usize) -> Option<&(Self::Item, GTree<Self>)> {
        return self.get_pair_by_inverted_index(self.len() - (1 + index));
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

fn div_ceil(p: usize, q: usize) -> usize {
    if p % q == 0 {
        return p / q;
    } else {
        return (p / q) + 1;
    }
}

pub fn physical_height<const K: usize, T: Clone + Ord + Debug>(
    t: &GTree<NonemptyReverseKList<K, T>>,
) -> usize {
    // println!("t: {:#?}", t);
    match t {
        GTree::Empty => return 0,
        GTree::NonEmpty(ref node) => {
            let len = node.set.len();
            // println!("len: {:#?}", len);
            let mut height = 0;

            for i in 0..len {
                let (_, subtree) = node.set.get_pair_by_index(i).unwrap();
                height = std::cmp::max(height, physical_height(subtree) + div_ceil(i, K));

                if i == len - 1 {
                    height = std::cmp::max(height, physical_height(&node.right) + div_ceil(i, K));
                }
            }

            return height;
        }
    }
}

// pub fn physical_height<const K: usize, T: Clone + Ord + Debug>(
//     t: &GTree<NonemptyReverseKList<K, T>>,
// ) -> usize {
//     // println!("t: {:#?}", t);
//     match t {
//         GTree::Empty => return 0,
//         GTree::NonEmpty(ref node) => {
//             let len = node.set.len();
//             // println!("len: {:#?}", len);
//             let mut height = 0;

//             let mut extra_height = 0;
//             for i in 0..len {
//                 if i % K == 0 {
//                     extra_height += 1;
//                 }

//                 let (_, subtree) = node.set.get_pair_by_index(i).unwrap();
//                 height = std::cmp::max(height, physical_height(subtree) + extra_height);

//                 if i == len - 1 {
//                     height = std::cmp::max(height, physical_height(&node.right) + extra_height);
//                 }
//             }

//             return height;
//         }
//     }
// }