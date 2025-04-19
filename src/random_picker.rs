#![allow(unused)]

use std::num::NonZeroUsize;

use crate::user::User;
use rand::Rng;
struct RandomPicker<'u, U> {
    binary_tree: Vec<BinaryTreeNode<'u, U>>,
}

#[derive(Debug)]
enum BinaryTreeNode<'u, U> {
    /// Either initialization stub, or `Leaf` user just got purged after `RandomPicker::draw_one`.
    /// See also `RandomPicker::compactify`
    None,
    /// Artifact of both `RandomPicker::draw_one` and `RandomPicker::compactify`:
    /// exactly one of its children is empty.
    /// Index would be maintained by `RandomPicker::draw_one` i.e. during the lottery,
    /// s.t. it saves us some jumps by directly leading us to the next interesting descendant
    One {
        descendant_idx: usize,
    },
    Two {
        sum: usize,
    },
    Leaf(&'u mut U),
}

impl<'u, U: for<'hrtb> User<'hrtb>> RandomPicker<'u, U> {
    fn left(u: usize) -> usize {
        u * 2 + 1
    }
    fn right(u: usize) -> usize {
        u * 2 + 2
    }
    fn parent(u: NonZeroUsize) -> usize {
        (u.get() - 1) / 2
    }
    fn bro(u: NonZeroUsize) -> NonZeroUsize {
        let u = u.get();
        NonZeroUsize::new(u - 1 + (u % 2) * 2).unwrap()
    }
    /// Make `RandomPicker::binary_tree` a _complete binary tree_,
    /// in which all internal nodes are `BinaryTreeNode::Two`,
    /// and all leaf nodes are `BinaryTreeNode::Leaf`
    pub(crate) fn new(iter: impl IntoIterator<Item = &'u mut U>) -> Self {
        let iter = iter.into_iter();
        let mut binary_tree = Vec::with_capacity(iter.size_hint().0 * 2);
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
        fn init_at<U: for<'u> User<'u>>(i: usize, v: &mut [BinaryTreeNode<U>]) {
            use crate::random_picker::RandomPicker as RP;
            if let BinaryTreeNode::None = &v[i] {
                // During initialization, `None` iff internal node,
                // and being a complete tree, internal nodes have both childrens,
                // thus here we may safely assume both left and right are still within boundary
                init_at(RP::<U>::left(i), v);
                init_at(RP::<U>::right(i), v);
                match (&v[RP::<U>::left(i)], &v[RP::<U>::right(i)]) {
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
    /// which is residual from `Self::compactify`,
    /// meaning exactly one of its left or right child/children is present,
    /// in which case we trim the tree via moving that remained child/children to this index,
    /// for if the child/children are lucky in the sense that one of them are the lucky user,
    /// this set of users probably contain a decent amount of tickets,
    /// s.t. this path is probably hot and would probably be walked again.
    ///
    /// Trimming is as simple as modifying the index to the first descendant that's not
    /// `BinaryTreeNode::One`.
    ///
    /// It might be the case some user might not admit any more prizes,
    /// in which case backtrace to the nearest parent of which both left and right are not empty,
    /// and use that other child.
    ///
    /// TODO
    /// implement entropy pool, maybe as simple as caching random numbers.
    fn draw_one(&mut self, rng: &mut impl Rng) {}

    /// Some nodes are not binary tree anymore: they have only one child.
    /// Trim them s.t. we don't have to traverse down the tree next time.
    ///
    /// N.B.
    /// The ticket counters are assumed to be valid and thus _not_ modified,
    /// i.e. the input index should point to `Option::<BinaryTreeNode>::None`
    fn compactify(&mut self, mut i: NonZeroUsize) {
        while let BinaryTreeNode::None = &self.binary_tree[i.get()] {
            match &mut self.binary_tree[Self::parent(i)] {
                BinaryTreeNode::None | BinaryTreeNode::Leaf(_) => panic!("Data strucutre inconsistent"),
                one @ BinaryTreeNode::One { .. } => {
                    // meaning the node was in fact a `BinaryTreeNode::Leaf`,
                    // i.e. we just removed a `&mut impl User` from the tree,
                    // thus this is where trim indeed happens
                    *one = BinaryTreeNode::None;
                },
                two @ &mut BinaryTreeNode::Two { sum } => {
                    todo!();
                },
            }
            if let Some(p) = NonZeroUsize::new(Self::parent(i)) {
                i = p;
            } else {
                break;
            }
        }
    }
}
