use std::collections::{HashMap, HashSet};
use std::hash::{Hash, RandomState};
use std::iter::repeat_n;

use rand::Rng;

use crate::entrant::Entrant;
use crate::prize::Prize;
use crate::space_efficient_shuffler;
pub struct Lottery<K, E, S = RandomState>
where
    K: Hash + Eq,
{
    /// Determine whether the lottery has been shuffled and done.
    shuffled: bool,
    /// The entrants in a hash map, use .entrants() to get the result
    entrants: HashMap<K, E, S>,
    /// The prizes, the lower the index, the higher the priority.
    prizes: Vec<Prize>,
}

impl<'entrant, K, E> Default for Lottery<K, E>
where
    K: Hash + Eq,
    E: Entrant<'entrant, Key = K>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'entrant, K, E> Lottery<K, E>
where
    K: Hash + Eq,
    E: Entrant<'entrant, Key = K>,
{
    pub fn new() -> Self {
        Self {
            entrants: HashMap::new(),
            prizes: Vec::new(),
            shuffled: false,
        }
    }

    pub fn with_entrant_capacity(cap: usize) -> Self {
        Self {
            entrants: HashMap::with_capacity(cap),
            prizes: Vec::new(),
            shuffled: false,
        }
    }
}

impl<'entrant, K, E, S> Lottery<K, E, S>
where
    K: Hash + Eq,
    E: Entrant<'entrant, Key = K>,
    S: std::hash::BuildHasher + std::default::Default,
{
    pub fn with_entrant_capacity_and_hasher(cap: usize, hasher: S) -> Self {
        Self {
            entrants: HashMap::with_capacity_and_hasher(cap, hasher),
            prizes: Vec::new(),
            shuffled: false,
        }
    }

    pub fn with_hasher(hasher: S) -> Self {
        Self {
            entrants: HashMap::with_hasher(hasher),
            prizes: Vec::new(),
            shuffled: false,
        }
    }

    /// Check if the prize has been distributed.
    pub fn shuffled(&self) -> bool {
        self.shuffled
    }

    /// Add a entrant to the lottery.
    /// When the keys collide, it would return the old entrant.
    pub fn add_entrant(&mut self, entrant: E) -> Option<E> {
        self.entrants.insert(entrant.key(), entrant)
    }

    /// Set all the entrants in the lottery.
    /// It is possible to have less entrant if the keys collide.
    pub fn set_entrants<C>(&mut self, entrants: C)
    where
        C: IntoIterator<Item = E>,
    {
        self.entrants.clear();
        self.entrants = entrants.into_iter().map(|u| (u.key(), u)).collect();
    }

    /// Add a prize to the lottery.
    /// The prior added prize would be considered as bigger prize.
    pub fn add_prize(&mut self, prize: Prize) {
        self.prizes.push(prize);
    }

    /// Set the whole prizes of the lottery.
    /// The prize in position 0 would be the biggest prize, while the last prize would be the
    /// smallest prize.
    pub fn set_prizes<C>(&mut self, prizes: C)
    where
        C: IntoIterator<Item = Prize>,
    {
        self.prizes.clear();
        self.prizes = prizes.into_iter().collect();
    }

    /// Returns the entrants.
    pub fn entrants(&self) -> std::collections::hash_map::Values<'_, K, E> {
        self.entrants.values()
    }

    /// Returns the entrants, which are mutable.
    pub fn entrants_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, K, E> {
        self.entrants.values_mut()
    }
}

impl<'entrant, 'prize, K, E, S> Lottery<K, E, S>
where
    K: Hash + Eq,
    E: Entrant<'entrant, Key = K>,
    S: std::hash::BuildHasher + std::default::Default,
    'prize: 'entrant,
{
    /// Shuffle and distribute the prizes to the entrants.
    pub fn shuffle(&'prize mut self, rng: &mut impl Rng) {
        // FIXME: this is just a guess.
        // Let `u` be the number of entrants, `t` be the number of tickets, `p` be the number of prizes.
        // Assuming the size of "tree" would require 24 * 2 * u * log(u),
        // while the size of "array" would require 16 * t.
        // We might compare `3 * u * log(u)` with `t`.
        // XXX: Engineering factor for adjusting the boundary.
        let array_est: usize = self.entrants.values().map(|u| u.ticket_count()).sum();
        let tree_est = 3 * self.entrants.len() * self.entrants.len().ilog2() as usize;
        let tree_factor = 1;
        if array_est <= tree_est * tree_factor {
            self.shuffle_array_inner(rng, array_est);
        } else {
            self.shuffle_tree(rng);
        }
    }

    /// Shuffle the branches and distribute the prizes to the entrants.
    pub fn shuffle_tree(&'prize mut self, rng: &mut impl Rng) {
        if self.shuffled {
            return;
        }
        self.shuffled = true;
        let mut prizes = self
            .prizes
            .iter()
            .flat_map(|p| repeat_n((), p.count()).map(move |_| p))
            .peekable();
        let mut space_efficient_shuffler =
            space_efficient_shuffler::SpaceEfficientShuffler::new(self.entrants.values_mut());
        while space_efficient_shuffler.try_draw_one(rng, &mut prizes) {}
    }

    /// Shuffle the slots and distribute the prizes to the entrants.
    pub fn shuffle_array<R>(&'prize mut self, rng: &mut R)
    where
        R: Rng,
    {
        let total_ticket_count: usize = self.entrants.values().map(|u| u.ticket_count()).sum();
        self.shuffle_array_inner(rng, total_ticket_count);
    }

    fn shuffle_array_inner<R>(&'prize mut self, rng: &mut R, num_tickets: usize)
    where
        R: Rng,
    {
        use std::iter::repeat_n;

        let keys = self.entrants.values().map(Entrant::key).collect::<Vec<_>>();
        let mut available_entrants = keys.iter().collect::<HashSet<&K, S>>();
        // Shuffle the slots, each entrant has `entrant.ticket_count()` slots.
        // Use the entrant's key to point back to itself.
        // The complexity shuffling would be both O(n).
        let tickets_god_only_knows_which_entrant = {
            use rand::seq::SliceRandom;
            let mut ret =
                self.entrants
                    .values()
                    .zip(&keys)
                    .fold(Vec::with_capacity(num_tickets), |mut ret, (entrant, key)| {
                        ret.extend(repeat_n(key, entrant.ticket_count()));
                        ret
                    });
            ret.shuffle(rng);
            ret
        };

        // Map the prize to entrant, the overall time complexity would be O(k)
        // once the key-mapped entrant has been saturated, try next entrant
        // For example, let's call `this.prizes[i]` `pi`, `this.entrants.keys()[i]` `ki`.
        // `p0.count = 1`, `p1.count = 2`, `p2.count = 3`.
        // We will have `prizes = [&p0, &p1, &p1, &p2, &p2, &p2]`
        // and the tickets would be like `[k5, k4, k4, k3, k2, k5, k6, k7...]`.
        // over iteration the mapping would be
        // p0 -> u5
        // p1 -> u4
        // p1 -> u3 // skip u4, assuming each entrant would only hold 1 prize
        // p2 -> u2
        // p2 -> u6 // skip u5, same assumption
        // p2 -> u7
        let prizes = self.prizes.iter().flat_map(|p| repeat_n(p, p.count()));
        let mut tickets = tickets_god_only_knows_which_entrant.into_iter();
        // I do hate this notation, but borrowing is annoier
        'outer: for prize in prizes {
            for ticket in tickets.by_ref() {
                // It is possible to use raw pointer to reduce both key production and hashing costs.
                // However, it requires unsafe.
                // Fortunately, this is not recursive, and relative simple to check the boundary.
                if available_entrants.contains(ticket) {
                    if self.entrants.get_mut(ticket).unwrap().add_prize(prize) {
                        // entrant may accept more prizes, no-op
                        break;
                    } else {
                        // entrant accepts prizes no more, purge it
                        available_entrants.remove(ticket);
                        if available_entrants.is_empty() {
                            // when all the entrants are fulfilled (none of them can add prize),
                            // it is good to early return.
                            break 'outer;
                        }
                    }
                }
            }
        }
        // always ensure the process won't proceed twice
        self.shuffled = true;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::Lottery;
    use crate::prize::PrizeBuilder;
    use crate::test_utils::GenericEntrant;

    #[test]
    fn test_space_efficient_shuffler() {
        let mut rng = rand::rng();
        const MAX_PRIZE_COUNT: usize = 100;
        const NUM_ENTRANTS: usize = 65536;
        let (prizes, num_prizes) = {
            let mut n = 0;
            let ret = Vec::from_iter((0..MAX_PRIZE_COUNT).map(|x| {
                n += x;
                PrizeBuilder::new().count(x).name(format!("{x}")).build()
            }));
            (ret, n)
        };
        let (entrants, log) = (0..NUM_ENTRANTS).map(GenericEntrant::new).collect::<(Vec<_>, Vec<_>)>();
        let mut tickets = Lottery::new();
        prizes.into_iter().for_each(|p| tickets.add_prize(p));
        entrants.into_iter().for_each(|u| {
            tickets.add_entrant(u);
        });
        tickets.shuffle_tree(&mut rng);
        assert_eq!(log.iter().map(|u| u.borrow().len()).sum::<usize>(), num_prizes);
        (0..MAX_PRIZE_COUNT).for_each(|i| {
            let name = &format!("{i}");
            assert_eq!(
                log.iter()
                    .map(|rc| &**rc)
                    .map(RefCell::borrow)
                    .map(|v| v.as_slice().iter().filter(|p| p.name() == name).count())
                    .sum::<usize>(),
                i
            )
        });
    }
}
