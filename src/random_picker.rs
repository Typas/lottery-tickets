#![allow(unused)]

use std::{iter::Peekable, num::NonZeroUsize};

use crate::{prize::Prize, user::User};
use rand::Rng;
pub(crate) struct RandomPicker<'u, U> {
    binary_tree: Vec<BinaryTreeNode<'u, U>>,
}

#[derive(Debug)]
enum BinaryTreeNode<'u, U> {
    /// Either initialization stub, or `Leaf` just got purged after `RandomPicker::draw_one`.
    /// See also `RandomPicker::cleanup_after_purge_node`
    None,
    /// Artifact of both `RandomPicker::draw_one` and `RandomPicker::cleanup_after_purge_node`:
    /// exactly one of its children is empty,
    /// the `descendant_idx` points us to the next interesting descendant.
    One {
        /// At times there might be subtree that form a pure linked list,
        /// comprised only of consecutive `One`, except the end node being `Two` or `Leaf`,
        /// in which case the `descendant_idx` shall only point to indices within the list.
        ///
        /// I.e. if there's some path comprised of consecutive `One`, then a `Two`,
        /// followed by more `One`,
        /// those `One` in the previous part shall not point any `One` in the latter part.
        descendant_idx: NonZeroUsize,
    },
    Two {
        sum: usize,
    },
    Leaf(&'u mut U),
}

impl<'u, U: for<'hrtb> User<'hrtb>> RandomPicker<'u, U> {
    const ERR_DATA_INCONSISTENT: &'static str = "Data strucutre inconsistent";

    fn left(u: usize) -> NonZeroUsize {
        NonZeroUsize::new(u * 2 + 1).unwrap()
    }
    fn right(u: usize) -> NonZeroUsize {
        NonZeroUsize::new(u * 2 + 2).unwrap()
    }
    fn parent(u: NonZeroUsize) -> usize {
        (u.get() - 1) / 2
    }
    fn sibling(u: NonZeroUsize) -> NonZeroUsize {
        let u = u.get();
        NonZeroUsize::new(u - 1 + (u % 2) * 2).unwrap()
    }

    /// Make `RandomPicker::binary_tree` a _complete binary tree_,
    /// in which all internal nodes are `BinaryTreeNode::Two`,
    /// and all leaf nodes are `BinaryTreeNode::Leaf`
    pub(crate) fn new(iter: impl IntoIterator<Item = &'u mut U>) -> Self {
        let iter = iter.into_iter();
        let mut binary_tree = Vec::with_capacity(iter.size_hint().0 * 2);
        binary_tree.extend(iter.map(BinaryTreeNode::Leaf));
        if binary_tree.is_empty() {
            // early return s.t. later we may assume non-zero user count
            return Self { binary_tree };
        } else {
            // to make a complete binary tree in which leaf node iff `BinaryTreeNode::Leaf`,
            // # of internal nodes is exactly one less than (# of leafs i.e. # of users)
            let mut internal_nodes =
                Vec::from_iter(std::iter::repeat_n((), binary_tree.len() - 1).map(|_| BinaryTreeNode::None));
            binary_tree = {
                internal_nodes.append(&mut binary_tree);
                internal_nodes
            };
        }

        /// Given internal nodes be `BinaryTreeNode::None` and leaf nodes `BinaryTreeNode::Leaf`,
        /// make every internal node `BinaryTreeNode::Two`:
        /// internal nodes in a complete binary tree all have two children
        ///
        /// Input should be valid indices only.
        fn init_at<U: for<'hrtb> User<'hrtb>>(i: usize, v: &mut [BinaryTreeNode<U>]) {
            use crate::random_picker::RandomPicker as RP;
            if let BinaryTreeNode::None = &v[i] {
                // During initialization, `None` iff internal node,
                // and being a complete tree, internal nodes have both childrens,
                // thus here we may safely assume both left and right are still within boundary
                init_at(RP::<U>::left(i).get(), v);
                init_at(RP::<U>::right(i).get(), v);
                match (&v[RP::<U>::left(i).get()], &v[RP::<U>::right(i).get()]) {
                    (BinaryTreeNode::Leaf(l), BinaryTreeNode::Leaf(r)) => {
                        v[i] = BinaryTreeNode::Two {
                            sum: l.ticket_count() + r.ticket_count(),
                        }
                    },
                    (BinaryTreeNode::Two { sum }, BinaryTreeNode::Leaf(u)) => {
                        v[i] = BinaryTreeNode::Two {
                            sum: sum + u.ticket_count(),
                        }
                    },
                    (BinaryTreeNode::Two { sum: l }, BinaryTreeNode::Two { sum: r }) => {
                        v[i] = BinaryTreeNode::Two { sum: l + r }
                    },
                    _ => {
                        panic!("Not a complete binary tree!");
                    },
                }
            }
        }

        // we've early return if there were no users in the first place
        init_at(0, &mut binary_tree);
        Self { binary_tree }
    }

    /// Each node is either one user or a set of users, as determined by `BinaryTreeNode`.
    ///
    /// Drawing a lucky user is done via traversing down the tree via sequence of binary questions,
    /// based on left/right tickets count,
    /// till a node that is exactly one user is found.
    ///
    /// During the walk till the lucky user, we may find `BinaryTreeNode::One`,
    /// which is residual from `Self::cleanup_after_purge_node`,
    /// meaning exactly one of its left or right child/children is present,
    /// in which case we may try jump to next "interesting" descendant
    /// and record where we jumped via modifying the `BinaryTreeNode::One::descendant_idx`,
    /// for if that descendant is lucky in the sense that one of them are the lucky user,
    /// this descendant probably contains a decent amount of tickets,
    /// s.t. this path is probably hot and would probably be walked again.
    ///
    /// Note it might be the case some user might not admit any more prizes,
    /// in which case we should retry from root.
    ///
    /// Return:
    /// `Err` if no more lottery either because no users or no prizes,
    /// `Ok(Some(..))` if successfully picked the lucky user,
    /// `Ok(None)` if spurious fail and we shall try again.
    ///
    /// TODO
    /// implement entropy pool, maybe as simple as caching random numbers.
    pub(crate) fn draw_one<'p>(
        &mut self,
        rng: &mut impl Rng,
        prizes: &mut Peekable<impl Iterator<Item = &'p Prize>>,
    ) -> Result<Option<&U>, ()> {
        let mut idx = 0;
        loop {
            // ugly indexing to circumvent `&mut` lifetime
            // TODO: refactor
            match &self.binary_tree[idx] {
                BinaryTreeNode::None => panic!("{}", Self::ERR_DATA_INCONSISTENT),
                BinaryTreeNode::One {
                    descendant_idx: nearer_descendant_idx,
                } => {
                    // perform the jump via preparing the index for next iteration
                    let mut new_idx = nearer_descendant_idx.get();
                    if let BinaryTreeNode::One {
                        descendant_idx: further_descendant_idx,
                    } = &self.binary_tree[nearer_descendant_idx.get()]
                    {
                        // turns out we may jump further, record this fact.
                        new_idx = further_descendant_idx.get();
                        self.binary_tree[idx] = BinaryTreeNode::One {
                            descendant_idx: *further_descendant_idx,
                        };
                    }
                    idx = new_idx;
                },
                BinaryTreeNode::Leaf(..) => {
                    let BinaryTreeNode::Leaf(u) = &mut self.binary_tree[idx] else {
                        panic!()
                    };
                    // if no prizes, just bail out with error
                    let prize = prizes.peek().ok_or(())?;
                    let ticket_count = u.ticket_count();
                    if ticket_count > 0 && u.add_prize(*prize) {
                        // only advance the iterator if we're sure `impl User` takes it just fine
                        prizes.next();
                        self.decrease_tickets_count(idx, 1);
                    } else {
                        // Assuming `User::add_prize` is monotone,
                        // in the sense once returned `false` it's always `false`,
                        // we may delete this user from the tree.
                        //
                        // Do _not_ advance the iterator else we'll lose some prizes
                        self.decrease_tickets_count(idx, ticket_count);
                        self.binary_tree[idx] = BinaryTreeNode::None;
                        if let Some(idx_of_purged_leaf) = NonZeroUsize::new(idx) {
                            self.cleanup_after_purge_node(idx_of_purged_leaf);
                        } else {
                            // the index is just zero, meaning the leaf is also root,
                            // no more users available, just return error
                            Err(())?
                        }
                    }
                },
                BinaryTreeNode::Two { sum } => {
                    let left = Self::left(idx);
                    let left_ticket_count = self.get_key_count_at_idx(left);
                    idx = if rng.random_range(..*sum) < left_ticket_count {
                        left.get()
                    } else {
                        Self::right(idx).get()
                    };
                },
            }
        }
    }

    fn get_key_count_at_idx(&self, mut idx: NonZeroUsize) -> usize {
        loop {
            match &self.binary_tree[idx.get()] {
                BinaryTreeNode::Two { sum } => break *sum,
                BinaryTreeNode::One { descendant_idx } => idx = *descendant_idx,
                BinaryTreeNode::None => panic!("{}", Self::ERR_DATA_INCONSISTENT),
                BinaryTreeNode::Leaf(u) => break u.ticket_count(),
            }
        }
    }

    /// Remove tickets from all the ancestor nodes,
    /// keeping the counters (`BinaryTreeNode::Two`) sane.
    ///
    /// Note the node at input index is excluded.
    fn decrease_tickets_count(&mut self, mut idx: usize, ticket_count: usize) {
        while let Some(parent) = NonZeroUsize::new(idx).map(Self::parent) {
            idx = parent;
            if let BinaryTreeNode::Two { sum } = &mut self.binary_tree[parent] {
                *sum -= ticket_count;
            }
        }
    }

    /// Some nodes are not binary tree anymore: they have only one child.
    /// Trim them s.t. we don't have to traverse down the tree next time.
    ///
    /// N.B.
    /// The input index should point to `BinaryTreeNode::None`.
    /// The ticket counters are assumed to be valid and thus _not_ modified.
    fn cleanup_after_purge_node(&mut self, mut i: NonZeroUsize) {
        use crate::random_picker::RandomPicker as RP;
        while let BinaryTreeNode::None = &self.binary_tree[i.get()] {
            match &mut self.binary_tree[Self::parent(i)] {
                BinaryTreeNode::None | BinaryTreeNode::Leaf(_) => panic!("{}", Self::ERR_DATA_INCONSISTENT),
                one @ BinaryTreeNode::One { .. } => {
                    // The parent `BinaryTreeNode::One` used to point to a `BinaryTreeNode::Leaf`,
                    // now it's `BinaryTreeNode::None`.
                    // Thus this parent should become `BinaryTreeNode::None`, too.
                    *one = BinaryTreeNode::None;
                    if let Some(p) = NonZeroUsize::new(Self::parent(i)) {
                        i = p;
                    } else {
                        break;
                    }
                },
                two @ &mut BinaryTreeNode::Two { .. } => {
                    // Lazy: we don't care if the sibling is `BinaryTreeNode::One`,
                    // for the `RandomPicker` would trim them as they see fit:
                    // if that sibling has few tickets, it might not be accessed ever again anyway
                    *two = BinaryTreeNode::One {
                        descendant_idx: RP::<U>::sibling(i),
                    };
                    break;
                },
            }
        }
    }
}
