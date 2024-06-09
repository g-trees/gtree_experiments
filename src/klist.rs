use std::{cmp::*, rc::Rc};

use crate::{Set, GTree, NonemptySet, NonemptySetMeta};

/// A k-list, except for a few optimization details:
///
/// - the list is known to be non-empty, so it always contains at least one vertex, and
/// - the list stores its items in reverse order, to enable `insert_min` and `remove_min` in O(1) rather than O(n) time.
#[derive(Clone)]
pub struct NonemptyReverseKList<const K: usize, I: Clone + Ord> {
    data: [Option<(I, GTree<Self>)>; K],
    next: Option<Rc<Self>>,
}

impl<const K: usize, I: Clone + Ord> NonemptyReverseKList<K, I> {
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

            // First, we pop off the first n items
            let removed: [_; K] = std::array::from_fn(|i| {
                if i < n {
                    return self.data[i].clone();
                } else {
                    return None;
                }
            });

            // Next, we move the remaining items of the current vertex to the front.
            let mut new_data: [_; K] = std::array::from_fn(|i| {
                if i + n < K {
                    return self.data[i + n].clone();
                } else {
                    return None;
                }
            });

            match self.next {
                None => {
                    // If we are the last vertex, we are done.
                    return (
                        removed,
                        Some(NonemptyReverseKList {
                            data: new_data,
                            next: None,
                        }),
                    );
                }
                Some(ref next) => {
                    // Recurse on the remaining vertices.
                    let (removed_rec, remaining_rec) = next.remove_n_max(n);

                    // Copy the recursively removed items into the new data for this vertex.
                    for i in 0..n {
                        new_data[n + i] = removed_rec[i].clone();
                    }

                    return (
                        removed,
                        Some(NonemptyReverseKList {
                            data: new_data,
                            next: remaining_rec.map(Rc::new),
                        }),
                    );
                }
            }
        }
    }

    // Internal helper function: get an item by index, where index 0 denotes the *greatest* item.
    fn get_by_inverted_index(&self, index: usize) -> Option<&I> {
        if index < K {
            return self.data[index].as_ref().map(|(item, _)| item);
        } else {
            match self.next {
                Some(ref next) => return next.get_by_inverted_index(index - K),
                None => return None,
            }
        }
    }
}

impl<const K: usize, I: Clone + Ord> NonemptySet for NonemptyReverseKList<K, I> {
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
                for i in K-1..=0 {
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
                        data: right_data,
                        next: None,
                    })
                };

                // We obtain the left return by removing our first `i` items.
                let (_, left) = self.remove_n_max(i);

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

                if i == K {
                    // We contain no item less than the key.

                    match self.next {
                        None => {
                            // All items are greater than the key.
                            return (Set::Empty, None, Set::NonEmpty(self.clone()));
                        }
                        Some(ref next) => {
                            // Recurse and join ourselves to the right of the right recursive return.
                            let (right_rec, mid_rec, left_rec) = next.split(key);

                            match right_rec {
                                Set::Empty => return (left_rec, mid_rec, Set::NonEmpty(self.clone())),
                                Set::NonEmpty(right_rec) => return (
                                    left_rec,
                                    mid_rec,
                                    Set::NonEmpty(Self::join(self, &right_rec)),
                                ),
                            }
                        }
                    }
                } else {
                    // i is less than K, so we contain i items greater than the key,
                    // and at least one item less than the key.

                    // First, we compute the right return.
                    let right = if i == 0 {
                        // The first item less than the key is the very first item, so the right return is the empty set.
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
                            data: right_data,
                            next: None,
                        })
                    };

                    // We obtain the left return by removing our first `i` items.
                    let (_, left) = self.remove_n_max(i);

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
        match left.next {
            Some(ref left_next) => {
                // Recurse and use the return value as the next vertex for the first vertex of `left`.
                return NonemptyReverseKList {
                    data: left.data.clone(),
                    next: Some(Rc::new(Self::join(left_next, right))),
                };
            }
            None => {
                // Actually need to do some work now.

                // How many items does `left` store?
                let mut left_count = 0;
                for i in 0..K {
                    match left.data[i] {
                        Some(_) => left_count += 1,
                        None => break,
                    }
                }

                if left_count == K {
                    // Left is full, so we can simply set left.next to right.
                    return NonemptyReverseKList {
                        data: left.data.clone(),
                        next: Some(Rc::new(right.clone())),
                    };
                } else {
                    // Left has K - left_count free slots, so move that many items from right into left, and then concatenate.
                    let to_move = K - left_count;
                    let (right_removed, right_remaining) = right.remove_n_max(to_move);
                    let new_data: [_; K] = std::array::from_fn(|i| {
                        if i < left_count {
                            return left.data[i].clone();
                        } else {
                            return right_removed[i - left_count].clone();
                        }
                    });
                    
                    return NonemptyReverseKList {
                        data: new_data,
                        next: right_remaining.map(|r| Rc::new(r)),
                    };
                }
            }
        }
    }
}

impl<const K: usize, I: Clone + Ord> NonemptySetMeta for NonemptyReverseKList<K, I> {
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
                for i in K-1..=0 {
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
                for i in 0..K {
                    if let Some(_) = self.data[i] {
                        return i;
                    }
                }
                unreachable!("self.data contains at least one item.");
            }
        }
    }

    fn get_by_index(&self, index: usize) -> Option<&Self::Item> {
        return self.get_by_inverted_index(self.len() - (1 + index));
    }
}