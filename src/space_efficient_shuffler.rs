use std::marker::PhantomData;
use std::{iter::Peekable, num::NonZeroUsize};

use crate::{prize::Prize, user::User};
use rand::Rng;
pub(crate) struct SpaceEfficientShuffler<'prize, 'u, U>
where
    'prize: 'u,
{
    binary_tree: Vec<BinaryTreeNode<'u, U>>,
    _marker: PhantomData<&'prize ()>,
}

#[derive(Debug)]
enum BinaryTreeNode<'u, U> {
    /// Either initialization stub, or `Leaf` just got purged after `SpaceEfficientShuffler::draw_one`.
    /// See also `SpaceEfficientShuffler::cleanup_after_purge_node`
    None,
    /// Artifact of both `SpaceEfficientShuffler::draw_one` and `SpaceEfficientShuffler::cleanup_after_purge_node`:
    /// exactly one of its children is `None`,
    /// and `descendant_idx` points us towards the next interesting descendant.
    One {
        /// At times there might be path in the tree that forms a linked list,
        /// in the sense that except for the tail they all have exactly one child that's not `None`,
        /// i.e. comprised only of consecutive `One` except the tail being `Two` or `Leaf`,
        /// in which case the `descendant_idx` shall only point to indices within the list.
        ///
        /// I.e. if there's some path comprised of consecutive `One`, then a `Two`,
        /// followed by more `One`,
        /// those `One` in the previous part shall not point to any `One` in the latter part.
        descendant_idx: NonZeroUsize,
    },
    /// This node has two children.
    Two {
        sum: usize,
    },
    Leaf(&'u mut U),
}

impl<'prize, 'u, U: User<'u>> SpaceEfficientShuffler<'prize, 'u, U>
where
    'prize: 'u,
{
    const ERR_DATA_INCONSISTENT: &'static str = "Data strucutre inconsistent";

    fn left(u: usize) -> NonZeroUsize {
        NonZeroUsize::new(u * 2 + 1).unwrap()
    }
    fn right(u: usize) -> NonZeroUsize {
        NonZeroUsize::new(u * 2 + 2).unwrap()
    }
    fn parent(u: usize) -> Option<usize> {
        u.checked_sub(1).map(|i| i / 2)
    }
    fn sibling(u: NonZeroUsize) -> NonZeroUsize {
        let u = u.get();
        NonZeroUsize::new(u - 1 + (u % 2) * 2).unwrap()
    }

    /// Make `SpaceEfficientShuffler::binary_tree` a _complete binary tree_,
    /// in which all internal nodes are `BinaryTreeNode::Two`,
    /// and all leaf nodes are `BinaryTreeNode::Leaf`
    pub(crate) fn new(iter: impl IntoIterator<Item = &'u mut U>) -> Self {
        let iter = iter.into_iter();
        let mut binary_tree = Vec::with_capacity(iter.size_hint().0 * 2);
        binary_tree.extend(iter.map(BinaryTreeNode::Leaf));
        if binary_tree.is_empty() {
            // early return s.t. later we may assume non-zero user count
            return Self {
                binary_tree,
                _marker: PhantomData,
            };
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
        fn init_at<'u, U: User<'u> + 'u>(i: usize, v: &mut [BinaryTreeNode<U>]) {
            use crate::space_efficient_shuffler::SpaceEfficientShuffler as SES;
            if let BinaryTreeNode::None = &v[i] {
                // During initialization, `None` iff internal node,
                // and being a complete tree, internal nodes have both childrens,
                // thus here we may safely assume both left and right are still within boundary
                init_at(SES::<U>::left(i).get(), v);
                init_at(SES::<U>::right(i).get(), v);
                match (&v[SES::<U>::left(i).get()], &v[SES::<U>::right(i).get()]) {
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
        Self {
            binary_tree,
            _marker: PhantomData,
        }
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
    pub(crate) fn try_draw_one(
        &mut self,
        rng: &mut impl Rng,
        prizes: &mut Peekable<impl Iterator<Item = &'u Prize>>,
    ) -> Result<Option<&U>, ()> {
        let mut idx = 0;
        loop {
            // ugly indexing to circumvent `&mut` lifetime
            // TODO: refactor
            match &self.binary_tree[idx] {
                BinaryTreeNode::None => Err(())?,
                BinaryTreeNode::One { descendant_idx } => {
                    let mut next_interesting_idx = descendant_idx.get();
                    while let BinaryTreeNode::One {
                        descendant_idx: further_descendant_idx,
                    } = &self.binary_tree[next_interesting_idx]
                    {
                        // try jump further
                        next_interesting_idx = further_descendant_idx.get();
                    }
                    self.binary_tree[idx] = BinaryTreeNode::One {
                        descendant_idx: NonZeroUsize::new(next_interesting_idx).unwrap(),
                    };
                    idx = next_interesting_idx;
                },
                BinaryTreeNode::Leaf(..) => {
                    let BinaryTreeNode::Leaf(u) = &mut self.binary_tree[idx] else {
                        panic!()
                    };
                    // if no prizes, just bail out with error
                    let prize = prizes.peek().ok_or(())?;
                    let ticket_count = u.ticket_count();
                    if ticket_count > 0 && u.add_prize(prize) {
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
                            self.cleanup_after_purge_node(idx_of_purged_leaf.get());
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
        while let Some(parent) = Self::parent(idx) {
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
    fn cleanup_after_purge_node(&mut self, mut i: usize) {
        use crate::space_efficient_shuffler::SpaceEfficientShuffler as SES;
        while let (Some(parent_idx), BinaryTreeNode::None) = (Self::parent(i), &self.binary_tree[i]) {
            match &mut self.binary_tree[parent_idx] {
                BinaryTreeNode::None | BinaryTreeNode::Leaf(_) => panic!("{}", Self::ERR_DATA_INCONSISTENT),
                one @ BinaryTreeNode::One { .. } => {
                    // The parent `BinaryTreeNode::One` used to point to either
                    // `BinaryTreeNode::One` or `BinaryTreeNode::Leaf`,
                    // but now the child is `BinaryTreeNode::None`.
                    // Thus this parent should become `BinaryTreeNode::None`, too.
                    *one = BinaryTreeNode::None;
                    i = parent_idx;
                },
                two @ &mut BinaryTreeNode::Two { .. } => {
                    // Lazy: we don't care what's the sibling,
                    // for the `SpaceEfficientShuffler::draw_one` would trim them as they see fit:
                    // if that sibling has few tickets, it might not be accessed ever again anyway
                    *two = BinaryTreeNode::One {
                        // `Option::unwrap` safety:
                        // this node has two children, meaning we must have sibling
                        descendant_idx: SES::<U>::sibling(NonZeroUsize::new(i).unwrap()),
                    };
                    break;
                },
            }
        }
    }
}
