use std::{
    iter::{Peekable, repeat},
    num::NonZeroUsize,
};

use crate::{prize::Prize, user::User};
use rand::Rng;
pub(crate) struct SpaceEfficientShuffler<'user, U> {
    /// A tree where being leaf iff `BinaryTreeNode::Leaf`, i.e. concrete user.
    ///
    /// All nodes are token for some subset of users.
    /// Children of a node are partitions of that node:
    /// they are disjoint subsets of the node and union of children is the node itself.
    ///
    /// See also `BinaryTreeNode`.
    binary_tree: Vec<BinaryTreeNode<'user, U>>,
}

#[derive(Debug)]
enum BinaryTreeNode<'u, U> {
    /// Either initialization stub, or `Leaf` just got purged after `SpaceEfficientShuffler::draw_one`.
    /// See also `SpaceEfficientShuffler::cleanup_after_purge_node`.
    /// A transient state: no valid node would be such that both its children are `None`.
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
        total_tickets_of_subtree: usize,
    },
    Leaf(&'u mut U),
}

impl<'prize, 'user, U: User<'prize> + 'prize> SpaceEfficientShuffler<'user, U>
where
    'prize: 'user,
{
    const ERR_DATA_INCONSISTENT: &'static str = "Data strucutre inconsistent";

    /// Use with caution: do not call on leaves!
    fn left(u: usize) -> NonZeroUsize {
        NonZeroUsize::new(u * 2 + 1).unwrap()
    }
    /// Use with caution: do not call on leaves!
    fn right(u: usize) -> NonZeroUsize {
        NonZeroUsize::new(u * 2 + 2).unwrap()
    }
    /// Return `None` if already root i.e. input `0`.
    fn parent(u: usize) -> Option<usize> {
        u.checked_sub(1).map(|i| i / 2)
    }
    /// Use with caution: is the sibling actually valid?
    fn sibling(u: NonZeroUsize) -> NonZeroUsize {
        let u = u.get();
        NonZeroUsize::new(u - 1 + (u % 2) * 2).unwrap()
    }

    /// Make `SpaceEfficientShuffler::binary_tree` a _complete binary tree_,
    /// in which all internal nodes are `BinaryTreeNode::Two`,
    /// and all leaf nodes are `BinaryTreeNode::Leaf`
    pub(crate) fn new(iter: impl IntoIterator<Item = &'user mut U>) -> Self {
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

        // complete binary tree: (# leaves) = L <=> (# nodes) = 2*L-1
        // e.g. (# leaves) = 1 => (# nodes) = 1
        //      (# leaves) = 2 => (# nodes) = 3
        //      (# leaves) = 3 => (# nodes) = 5
        //      ...
        // thus last internal node has index one less than length divided by two.
        (0..binary_tree.len() / 2).rev().for_each(|idx_internal_node| {
            match (
                &binary_tree[Self::left(idx_internal_node).get()],
                &binary_tree[Self::right(idx_internal_node).get()],
            ) {
                (BinaryTreeNode::Leaf(l), BinaryTreeNode::Leaf(r)) => {
                    binary_tree[idx_internal_node] = BinaryTreeNode::Two {
                        total_tickets_of_subtree: l.ticket_count() + r.ticket_count(),
                    }
                },
                (
                    BinaryTreeNode::Two {
                        total_tickets_of_subtree: sum,
                    },
                    BinaryTreeNode::Leaf(u),
                ) => {
                    binary_tree[idx_internal_node] = BinaryTreeNode::Two {
                        total_tickets_of_subtree: sum + u.ticket_count(),
                    }
                },
                (
                    BinaryTreeNode::Two {
                        total_tickets_of_subtree: l,
                    },
                    BinaryTreeNode::Two {
                        total_tickets_of_subtree: r,
                    },
                ) => {
                    binary_tree[idx_internal_node] = BinaryTreeNode::Two {
                        total_tickets_of_subtree: l + r,
                    }
                },
                _ => panic!("{}", Self::ERR_DATA_INCONSISTENT),
            }
        });

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
    /// `false` if no more lottery can be drawn, either because of no remaining users or no remaining prizes;
    /// `true` if successfully picked the lucky user.
    ///
    /// TODO
    /// implement entropy pool, maybe as simple as caching random numbers.
    /// comment: if you want to implement an entropy pool,
    /// it would be better to placed in `Tickets`,
    /// and fetch some random numbers during adding things.
    /// However, how could you use it when the rng is outsourced?
    pub(crate) fn try_draw_one(
        &mut self,
        rng: &mut impl Rng,
        prizes: &mut Peekable<impl Iterator<Item = &'prize Prize>>,
    ) -> bool {
        // if no prizes, just bail out with error
        let Some(prize) = prizes.peek() else { return false };

        let mut idx = 0;
        loop {
            // ugly indexing to circumvent `&mut` lifetime
            // TODO: refactor
            match &self.binary_tree[idx] {
                BinaryTreeNode::None => panic!("{}", Self::ERR_DATA_INCONSISTENT),
                BinaryTreeNode::One { descendant_idx } => {
                    let mut next_interesting_idx = descendant_idx.get();
                    while let BinaryTreeNode::One { descendant_idx } = &self.binary_tree[next_interesting_idx] {
                        // try jump further
                        next_interesting_idx = descendant_idx.get();
                    }
                    self.binary_tree[idx] = BinaryTreeNode::One {
                        descendant_idx: NonZeroUsize::new(next_interesting_idx).unwrap(),
                    };
                    idx = next_interesting_idx;
                },
                BinaryTreeNode::Two {
                    total_tickets_of_subtree,
                } => {
                    let left = Self::left(idx);
                    let left_ticket_count = self.get_key_count_at_idx(left);
                    // We're relying on the tree invariant that a `BinaryTreeNode::Two` node
                    // contains the total sum of tickets of both of its children.
                    // (See also `SpaceEfficientShuffler` and `BinaryTreeNode::Two`)
                    //
                    // Say left has L tickets, right has R tickets, we know `sum` is just (L+R),
                    // then a fair choice is simply generate a random number G (uniformly) between 0 and (L+R),
                    // and if G less than L, we choose the left subset,
                    // otherwise we choose the right subset.
                    idx = if rng.random_range(..*total_tickets_of_subtree) < left_ticket_count {
                        left.get()
                    } else {
                        Self::right(idx).get()
                    };
                },
                BinaryTreeNode::Leaf(..) => {
                    let BinaryTreeNode::Leaf(u) = &mut self.binary_tree[idx] else {
                        panic!()
                    };
                    let ticket_count = u.ticket_count();
                    if ticket_count > 0 && u.add_prize(prize) {
                        // only advance the iterator if we're sure `impl User` takes it just fine
                        prizes.next();
                        self.decrease_tickets_count(idx, 1);
                        return true;
                    } else {
                        // Assuming `User::add_prize` is monotone,
                        // in the sense once returned `false` it's always `false`,
                        // we may delete this user from the tree.
                        //
                        // Do _not_ advance the iterator else we'll lose some prizes
                        self.decrease_tickets_count(idx, ticket_count);
                        self.binary_tree[idx] = BinaryTreeNode::None;
                        self.cleanup_after_purge_node(idx);
                        if let BinaryTreeNode::None = &self.binary_tree[0] {
                            // we've depleted the users/tickets;
                            // bail out since we don't allow root to be
                            // `BinaryTreeNode::None`,
                            // but we're in an awkward case caused by either
                            // 1. Tree has exactly one node s.t. root is leaf
                            // 2. `SpaceEfficientShuffler::cleanup_after_purge_node`
                            return false;
                        } else {
                            // restart from root all over again,
                            // for all the probabilities based on which we choose path
                            // traversing down the tree are wrong.
                            idx = 0;
                            continue; // not necessary, just clarifying retrying from root
                        }
                    }
                },
            }
        }
    }

    fn get_key_count_at_idx(&self, mut idx: NonZeroUsize) -> usize {
        loop {
            match &self.binary_tree[idx.get()] {
                BinaryTreeNode::Two {
                    total_tickets_of_subtree: sum,
                } => break *sum,
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
    fn decrease_tickets_count(&mut self, idx: usize, ticket_count: usize) {
        repeat(())
            .scan(idx, |current_idx, _| {
                Self::parent(*current_idx).inspect(|parent_idx| *current_idx = *parent_idx)
            })
            .for_each(|idx| {
                if let BinaryTreeNode::Two {
                    total_tickets_of_subtree,
                } = &mut self.binary_tree[idx]
                {
                    *total_tickets_of_subtree -= ticket_count;
                }
            });
    }

    /// Some nodes are not binary tree anymore: they have only one child.
    /// Trim them s.t. we don't have to traverse down the tree next time.
    ///
    /// N.B.
    /// The input index should point to `BinaryTreeNode::None`.
    /// The ticket counters are assumed to be valid and thus _not_ modified.
    fn cleanup_after_purge_node(&mut self, mut i: usize) {
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
                        descendant_idx: Self::sibling(NonZeroUsize::new(i).unwrap()),
                    };
                    break;
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::SpaceEfficientShuffler;
    use crate::prize::PrizeBuilder;
    use crate::test_utils::CapacityOneUser;
    use crate::user::User;

    #[test]
    fn test_space_efficient_shuffler_few_users() {
        let mut rng = rand::rng();
        const MAX_PRIZE_COUNT: usize = 100;
        const NUM_USERS: usize = 1;
        let prizes =
            Vec::from_iter((0..MAX_PRIZE_COUNT).map(|x| PrizeBuilder::new().count(x).name(format!("{x}")).build()));
        {
            // A single user which doesn't hold any tickets
            let (mut users, log) = (0..NUM_USERS).map(CapacityOneUser::new).collect::<(Vec<_>, Vec<_>)>();
            let mut ses = SpaceEfficientShuffler::new(&mut users);
            let mut prizes = prizes.iter().peekable();
            let num_iterations = (1..)
                .take_while(|_| ses.try_draw_one(&mut rng, &mut prizes))
                .last()
                .unwrap_or(0);
            assert!(log.into_iter().all(|prizes_of_user| prizes_of_user.borrow().is_empty()));
            // draw, discovering that the user invalid, abort.
            // this shows when few users/tickets, we finish quickly
            assert_eq!(num_iterations, 0);
        }
        {
            // A single user which holds exactly one ticket
            let (mut users, log) = (1..=NUM_USERS).map(CapacityOneUser::new).collect::<(Vec<_>, Vec<_>)>();
            let mut ses = SpaceEfficientShuffler::new(&mut users);
            let mut prizes = prizes.iter().peekable();
            let num_iterations = (1..)
                .take_while(|_| ses.try_draw_one(&mut rng, &mut prizes))
                .last()
                .unwrap_or(0);
            assert_eq!(
                log.into_iter()
                    .map(|prizes_of_user| prizes_of_user.borrow().len())
                    .sum::<usize>(),
                1
            );
            // draw, ok.
            // draw, discovering that the user invalid, abort.
            // this shows when few users/tickets, we finish quickly
            assert_eq!(num_iterations, 1)
        }
    }

    #[test]
    fn test_space_efficient_shuffler_capacity_one_user() {
        let mut rng = rand::rng();
        const MAX_PRIZE_COUNT: usize = 100;
        const NUM_USERS: usize = 65536;
        let prizes =
            Vec::from_iter((0..MAX_PRIZE_COUNT).map(|x| PrizeBuilder::new().count(x).name(format!("{x}")).build()));
        let (mut users, log) = (0..NUM_USERS).map(CapacityOneUser::new).collect::<(Vec<_>, Vec<_>)>();
        let mut ses = SpaceEfficientShuffler::new(&mut users);
        let mut prizes = prizes.iter().peekable();
        while ses.try_draw_one(&mut rng, &mut prizes) {}
        assert_eq!(
            BTreeSet::from_iter(log.into_iter().flat_map(|prizes_of_user| {
                let prizes_of_user = prizes_of_user.borrow();
                assert!(prizes_of_user.len() <= 1);
                prizes_of_user.iter().next().map(|p| p.name()).map(String::from)
            })),
            BTreeSet::from_iter((0..MAX_PRIZE_COUNT).map(|x| format!("{x}")))
        );
    }

    #[test]
    fn test_space_efficient_shuffler_skewed_tickets() {
        const MAX_PRIZE_COUNT: usize = 10;
        const NUM_USERS: usize = MAX_PRIZE_COUNT + 1;
        const NORMAL_USER_TICKET_COUNT: usize = 1000;
        let mut rng = rand::rngs::mock::StepRng::new(1, 0); // this would always points to the rightmost in the tree
        // let mut rng = rand::rng(); // this would almost always success
        let leftmost_index = 5;
        let prizes =
            Vec::from_iter((0..MAX_PRIZE_COUNT).map(|x| PrizeBuilder::new().count(1).name(format!("{x}")).build()));
        // set the first user would always not have the prize
        let (mut users, log) = (0..NUM_USERS)
            .map(|u| {
                CapacityOneUser::with_tickets_count(
                    u,
                    if u == leftmost_index {
                        1
                    } else {
                        NORMAL_USER_TICKET_COUNT
                    },
                )
            })
            .collect::<(Vec<_>, Vec<_>)>();
        assert_eq!(
            users.iter().map(|u| u.ticket_count()).sum::<usize>(),
            (NUM_USERS - 1) * NORMAL_USER_TICKET_COUNT + 1
        );
        assert_eq!(users.iter().filter(|u| u.has_prize()).count(), 0);
        assert_eq!(prizes.iter().map(|p| p.count()).sum::<usize>(), MAX_PRIZE_COUNT);
        assert_eq!(users.len(), NUM_USERS);
        {
            let mut ses = SpaceEfficientShuffler::new(&mut users);
            assert_eq!(ses.binary_tree.len(), NUM_USERS * 2 - 1);
            let mut prizes = prizes.iter().peekable();
            while ses.try_draw_one(&mut rng, &mut prizes) {}
            assert_eq!(
                BTreeSet::from_iter(log.into_iter().flat_map(|prizes_of_user| {
                    let prizes_of_user = prizes_of_user.borrow();
                    assert!(prizes_of_user.len() <= 1);
                    prizes_of_user.iter().next().map(|p| p.name()).map(String::from)
                })),
                BTreeSet::from_iter((0..MAX_PRIZE_COUNT).map(|x| format!("{x}")))
            );
        }
        assert_eq!(users.len(), NUM_USERS);
        assert_eq!(users.iter().filter(|u| u.has_prize()).count(), NUM_USERS - 1);
        // in theory, this order is fixed
        // for (i, p) in prizes.iter().enumerate() {
        //     assert_eq!(users.iter().find(|u| u.prize == Some(p)).unwrap().key(), prize_order(i));
        // }
        assert!(users.iter().find(|u| u.key() == leftmost_index).is_some());
        assert_eq!(users.iter().find(|u| !u.has_prize()).unwrap().key(), leftmost_index);
        assert_eq!(
            users.iter().find(|u| u.key() == leftmost_index).unwrap().has_prize(),
            false
        );
    }

    // fn prize_order(i: usize) -> usize {
    //     i
    // }
}
