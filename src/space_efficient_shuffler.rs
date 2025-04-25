use std::{
    iter::{Peekable, repeat},
    num::NonZeroUsize,
};

use crate::{entrant::Entrant, prize::Prize};
use rand::Rng;
pub(crate) struct SpaceEfficientShuffler<'e, E> {
    /// A tree where being leaf iff `BinaryTreeNode::Leaf`, i.e. concrete entrant.
    ///
    /// All nodes are tokens for some subset of entrants.
    /// Children of a node are partitions of that node:
    /// they are disjoint subsets of the node and union of children is the node itself.
    ///
    /// See also `BinaryTreeNode`.
    binary_tree: Vec<BinaryTreeNode<'e, E>>,
}

#[derive(Debug)]
enum BinaryTreeNode<'e, E> {
    /// Either initialization stub, or `Leaf` just got purged after `SpaceEfficientShuffler::draw_one`.
    /// See also `SpaceEfficientShuffler::trim`.
    /// A transient state: no valid node would be such that both its children are `None`.
    None,
    /// Artifact of both `SpaceEfficientShuffler::draw_one` and `SpaceEfficientShuffler::trim`:
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
    Two { total_tickets_of_subtree: NonZeroUsize },
    /// We store only those entrants who have tickets.
    Leaf {
        entrant: &'e mut E,
        ticket_count: NonZeroUsize,
    },
}

impl<'prize, 'entrant, U: Entrant<'prize> + 'prize> SpaceEfficientShuffler<'entrant, U>
where
    'prize: 'entrant,
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
    pub(crate) fn new(iter: impl IntoIterator<Item = &'entrant mut U>) -> Self {
        let iter = iter.into_iter();
        let mut binary_tree = Vec::with_capacity(iter.size_hint().0 * 2);
        binary_tree.extend(iter.flat_map(|u| {
            NonZeroUsize::new(u.ticket_count()).map(|ticket_count| BinaryTreeNode::Leaf {
                entrant: u,
                ticket_count,
            })
        }));
        if binary_tree.is_empty() {
            // early return s.t. later we may assume non-zero entrant count
            return Self { binary_tree };
        } else {
            // to make a complete binary tree in which leaf node iff `BinaryTreeNode::Leaf`,
            // # of internal nodes is exactly one less than (# of leafs i.e. # of entrants)
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
                (BinaryTreeNode::Leaf { ticket_count: l, .. }, BinaryTreeNode::Leaf { ticket_count: r, .. }) => {
                    binary_tree[idx_internal_node] = BinaryTreeNode::Two {
                        total_tickets_of_subtree: l.checked_add(r.get()).unwrap(),
                    }
                },
                (
                    BinaryTreeNode::Two {
                        total_tickets_of_subtree: sum,
                    },
                    BinaryTreeNode::Leaf { ticket_count: r, .. },
                ) => {
                    binary_tree[idx_internal_node] = BinaryTreeNode::Two {
                        total_tickets_of_subtree: sum.checked_add(r.get()).unwrap(),
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
                        total_tickets_of_subtree: l.checked_add(r.get()).unwrap(),
                    }
                },
                _ => panic!("{}", Self::ERR_DATA_INCONSISTENT),
            }
        });

        Self { binary_tree }
    }

    /// Each node is either one entrant or a set of entrants, as determined by `BinaryTreeNode`.
    ///
    /// Drawing a lucky entrant is done via traversing down the tree via sequence of binary questions,
    /// based on left/right tickets count,
    /// till a node that is exactly one entrant is found.
    ///
    /// During the walk till the lucky entrant, we may find `BinaryTreeNode::One`,
    /// which is residual from `Self::trim`,
    /// meaning exactly one of its left or right child/children is present,
    /// in which case we may try jump to next "interesting" descendant
    /// and record where we jumped via modifying the `BinaryTreeNode::One::descendant_idx`,
    /// for if that descendant is lucky in the sense that one of them are the lucky entrant,
    /// this descendant probably contains a decent amount of tickets,
    /// s.t. this path is probably hot and would probably be walked again.
    ///
    /// Note it might be the case some entrant might not admit any more prizes,
    /// in which case we should retry from root.
    ///
    /// Return:
    /// `false` if no more lottery can be drawn, either because of no remaining entrants or no remaining prizes;
    /// `true` if successfully picked the lucky entrant.
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
        // remember to advance the iterator if some lucky entrant were found
        let Some(prize) = prizes.peek() else { return false };
        // if no entrants/tickets, again bail out with error
        if self.binary_tree.is_empty() {
            return false;
        };

        let mut idx = 0;
        loop {
            // ugly indexing to circumvent `&mut` lifetime
            // TODO: refactor
            match &self.binary_tree[idx] {
                BinaryTreeNode::None if idx == 0 => {
                    // This only happens if all the entrants run out of tickets,
                    // caused by `Self::trim`.
                    return false;
                },
                BinaryTreeNode::None => {
                    // `BinaryTreeNode::None` should be transient:
                    // they shall be absent from this function
                    panic!("{}", Self::ERR_DATA_INCONSISTENT);
                },
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
                    // Say left has L tickets, right has R tickets, we know `total_tickets_of_subtree` is just (L+R),
                    // then a fair choice is simply generate a random number G (uniformly) between 0 and (L+R),
                    // and if G less than L, we choose the left subset,
                    // otherwise we choose the right subset.
                    idx = if rng.random_range(..total_tickets_of_subtree.get()) < left_ticket_count.get() {
                        left.get()
                    } else {
                        Self::right(idx).get()
                    };
                },
                BinaryTreeNode::Leaf { .. } => {
                    let &mut BinaryTreeNode::Leaf {
                        ref mut entrant,
                        ticket_count,
                        ..
                    } = &mut self.binary_tree[idx]
                    else {
                        unreachable!()
                    };
                    if entrant.add_prize(prize) {
                        prizes.next();
                        self.decrease_tickets_count(idx, NonZeroUsize::new(1).unwrap());
                        return true;
                    } else {
                        // Assuming `Entrant::add_prize` is monotone,
                        // in the sense once returned `false` it's always `false`,
                        // we may delete this entrant from the tree.
                        //
                        // Do _not_ advance the iterator else we'll lose some prizes
                        self.decrease_tickets_count(idx, ticket_count);
                        // restart from root all over again,
                        // for all the probabilities based on which we chose
                        // this exact path traversing down the tree are wrong.
                        idx = 0;
                        continue; // not necessary, just clarifying retrying from root
                    }
                },
            }
        }
    }

    fn get_key_count_at_idx(&self, mut idx: NonZeroUsize) -> NonZeroUsize {
        loop {
            match &self.binary_tree[idx.get()] {
                BinaryTreeNode::Two {
                    total_tickets_of_subtree: sum,
                } => break *sum,
                BinaryTreeNode::One { descendant_idx } => idx = *descendant_idx,
                BinaryTreeNode::None => panic!("{}", Self::ERR_DATA_INCONSISTENT),
                BinaryTreeNode::Leaf { ticket_count, .. } => break *ticket_count,
            }
        }
    }

    /// Remove tickets from all the ancestor nodes including self,
    /// keeping the counters (`BinaryTreeNode::Two`) sane.
    ///
    /// Input index must be `BinaryTreeNode::Leaf`.
    fn decrease_tickets_count(&mut self, idx: usize, decrement: NonZeroUsize) {
        repeat(())
            .scan(idx, |jdx, _| {
                // the input index should be taken care of
                let current_idx = *jdx;
                let parent_idx = Self::parent(*jdx).inspect(|parrent_idx| *jdx = *parrent_idx);
                match &mut self.binary_tree[current_idx] {
                    BinaryTreeNode::None => panic!("{}", Self::ERR_DATA_INCONSISTENT),
                    BinaryTreeNode::One { .. } => {},
                    BinaryTreeNode::Two {
                        total_tickets_of_subtree,
                    } => {
                        *total_tickets_of_subtree = total_tickets_of_subtree
                            .get()
                            .checked_sub(decrement.get())
                            .and_then(NonZeroUsize::new)
                            .unwrap();
                    },
                    BinaryTreeNode::Leaf { ticket_count, .. } => {
                        if let Some(t_c) = ticket_count
                            .get()
                            .checked_sub(decrement.get())
                            .and_then(NonZeroUsize::new)
                        {
                            *ticket_count = t_c;
                        } else {
                            self.binary_tree[current_idx] = BinaryTreeNode::None;
                        }
                    },
                }
                parent_idx
            })
            // work is done in `Iterator::scan`; still, we need to consume the iterator
            .count();

        if let BinaryTreeNode::None = &self.binary_tree[idx] {
            self.trim(idx);
        }
    }

    /// New `BinaryTreeNode::None` is produced as a side product of `Self::decrease_tickets_count`;
    /// update the ancestors, i.e.
    /// `BinaryTreeNode::Two` -> `BinaryTreeNode::One`,
    /// `BinaryTreeNode::One` -> `BinaryTreeNode::None`,
    /// if appropriate.
    ///
    /// N.B.
    /// The ticket counters are assumed to be valid and thus _not_ modified.
    /// Input must be valid index.
    fn trim(&mut self, mut i: usize) {
        while let (BinaryTreeNode::None, Some(parent_idx)) = (&self.binary_tree[i], Self::parent(i)) {
            match &mut self.binary_tree[parent_idx] {
                BinaryTreeNode::Leaf { .. } | BinaryTreeNode::None => panic!("{}", Self::ERR_DATA_INCONSISTENT),
                one @ BinaryTreeNode::One { .. } => {
                    // The parent `BinaryTreeNode::One` used to point to either
                    // `BinaryTreeNode::One` or `BinaryTreeNode::Leaf`,
                    // but now the child is `BinaryTreeNode::None`.
                    // Thus this parent should become `BinaryTreeNode::None`, too.
                    *one = BinaryTreeNode::None;
                    i = parent_idx;
                },
                two @ BinaryTreeNode::Two { .. } => {
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

    use rand::SeedableRng;

    use super::SpaceEfficientShuffler;
    use crate::entrant::Entrant;
    use crate::prize::PrizeBuilder;
    use crate::test_utils::CapacityOneEntrant;

    #[test]
    fn test_space_efficient_shuffler_few_entrants() {
        let mut rng = rand::rng();
        const MAX_PRIZE_COUNT: usize = 100;
        const NUM_ENTRANTS: usize = 1;
        let prizes =
            Vec::from_iter((0..MAX_PRIZE_COUNT).map(|x| PrizeBuilder::new().count(x).name(format!("{x}")).build()));
        {
            // A single entrant which doesn't hold any tickets
            let (mut entrants, log) = (0..NUM_ENTRANTS)
                .map(CapacityOneEntrant::new)
                .collect::<(Vec<_>, Vec<_>)>();
            let mut ses = SpaceEfficientShuffler::new(&mut entrants);
            let mut prizes = prizes.iter().peekable();
            let num_iterations = (1..)
                .take_while(|_| ses.try_draw_one(&mut rng, &mut prizes))
                .last()
                .unwrap_or(0);
            assert!(
                log.into_iter()
                    .all(|prizes_of_entrant| prizes_of_entrant.borrow().is_empty())
            );
            // draw, discovering that the entrant invalid, abort.
            // this shows when few entrants/tickets, we finish quickly
            assert_eq!(num_iterations, 0);
        }
        {
            // A single entrant which holds exactly one ticket
            let (mut entrants, log) = (1..=NUM_ENTRANTS)
                .map(CapacityOneEntrant::new)
                .collect::<(Vec<_>, Vec<_>)>();
            let mut ses = SpaceEfficientShuffler::new(&mut entrants);
            let mut prizes = prizes.iter().peekable();
            let num_iterations = (1..)
                .take_while(|_| ses.try_draw_one(&mut rng, &mut prizes))
                .last()
                .unwrap_or(0);
            assert_eq!(
                log.into_iter()
                    .map(|prizes_of_entrant| prizes_of_entrant.borrow().len())
                    .sum::<usize>(),
                1
            );
            // draw, ok.
            // draw, discovering that the entrant invalid, abort.
            // this shows when few entrants/tickets, we finish quickly
            assert_eq!(num_iterations, 1)
        }
    }

    #[test]
    fn test_space_efficient_shuffler_capacity_one_entrant() {
        let mut rng = rand::rng();
        const MAX_PRIZE_COUNT: usize = 100;
        const NUM_ENTRANTS: usize = 65536;
        let prizes =
            Vec::from_iter((0..MAX_PRIZE_COUNT).map(|x| PrizeBuilder::new().count(x).name(format!("{x}")).build()));
        let (mut entrants, log) = (0..NUM_ENTRANTS)
            .map(CapacityOneEntrant::new)
            .collect::<(Vec<_>, Vec<_>)>();
        let mut ses = SpaceEfficientShuffler::new(&mut entrants);
        let mut prizes = prizes.iter().peekable();
        while ses.try_draw_one(&mut rng, &mut prizes) {}
        assert_eq!(
            BTreeSet::from_iter(log.into_iter().flat_map(|prizes_of_entrant| {
                let prizes_of_entrant = prizes_of_entrant.borrow();
                assert!(prizes_of_entrant.len() <= 1);
                prizes_of_entrant.iter().next().map(|p| p.name()).map(String::from)
            })),
            BTreeSet::from_iter((0..MAX_PRIZE_COUNT).map(|x| format!("{x}")))
        );
    }

    #[test]
    fn test_space_efficient_shuffler_skewed_tickets() {
        const MAX_PRIZE_COUNT: usize = 10;
        const NUM_ENTRANTS: usize = MAX_PRIZE_COUNT + 1;
        const NORMAL_ENTRANT_TICKET_COUNT: usize = 1000;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1); // use small rng with seed to guarantee(?) the result would be the same.
        let poor_index = 0;
        let prizes =
            Vec::from_iter((0..MAX_PRIZE_COUNT).map(|x| PrizeBuilder::new().count(1).name(format!("{x}")).build()));
        // set the first entrant would always not have the prize
        let (mut entrants, log) = (0..NUM_ENTRANTS)
            .map(|u| {
                CapacityOneEntrant::with_tickets_count(
                    u,
                    if u == poor_index {
                        1
                    } else {
                        NORMAL_ENTRANT_TICKET_COUNT
                    },
                )
            })
            .collect::<(Vec<_>, Vec<_>)>();
        assert_eq!(
            entrants.iter().map(|u| u.ticket_count()).sum::<usize>(),
            (NUM_ENTRANTS - 1) * NORMAL_ENTRANT_TICKET_COUNT + 1
        );
        assert_eq!(entrants.iter().filter(|u| u.has_prize()).count(), 0);
        assert_eq!(prizes.iter().map(|p| p.count()).sum::<usize>(), MAX_PRIZE_COUNT);
        assert_eq!(entrants.len(), NUM_ENTRANTS);
        {
            let mut ses = SpaceEfficientShuffler::new(&mut entrants);
            assert_eq!(ses.binary_tree.len(), NUM_ENTRANTS * 2 - 1);
            let mut prizes = prizes.iter().peekable();
            while ses.try_draw_one(&mut rng, &mut prizes) {}
            assert_eq!(
                BTreeSet::from_iter(log.into_iter().flat_map(|prizes_of_entrant| {
                    let prizes_of_entrant = prizes_of_entrant.borrow();
                    assert!(prizes_of_entrant.len() <= 1);
                    prizes_of_entrant.iter().next().map(|p| p.name()).map(String::from)
                })),
                BTreeSet::from_iter((0..MAX_PRIZE_COUNT).map(|x| format!("{x}")))
            );
        }
        assert_eq!(entrants.len(), NUM_ENTRANTS);
        assert_eq!(entrants.iter().filter(|u| u.has_prize()).count(), NUM_ENTRANTS - 1);
        assert!(entrants.iter().any(|u| u.key() == poor_index));
        assert_eq!(entrants.iter().find(|u| !u.has_prize()).unwrap().key(), poor_index);
        assert!(!entrants.iter().find(|u| u.key() == poor_index).unwrap().has_prize());
    }
}
