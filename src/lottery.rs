use std::{
    collections::HashMap,
    hash::{Hash, RandomState},
    iter::repeat_n,
};

use rand::Rng;

use crate::{entrant::Entrant, prize::Prize, space_efficient_shuffler};
pub struct Lottery<K, U, S = RandomState>
where
    K: Hash + Eq,
{
    /// Determine whether the lottery has been shuffled and done.
    shuffled: bool,
    /// The entrants in a hash map, use .entrants() to get the result
    entrants: HashMap<K, U, S>,
    /// The prizes, the lower the index, the higher the priority.
    prizes: Vec<Prize>,
}

impl<'u, K, U> Default for Lottery<K, U>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'u, K, U> Lottery<K, U>
where
    K: Hash + Eq,
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

impl<'u, K, U, S> Lottery<K, U, S>
where
    K: Hash + Eq,
    U: Entrant<'u, Key = K>,
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
    pub fn add_entrant(&mut self, entrant: U) -> Option<U> {
        self.entrants.insert(entrant.key(), entrant)
    }

    /// Set all the entrants in the lottery.
    /// It is possible to have less entrant if the keys collide.
    pub fn set_entrants<C>(&mut self, entrants: C)
    where
        C: IntoIterator<Item = U>,
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
    /// The prize in position 0 would be the biggest prize, while the last prize would be the smallest prize.
    pub fn set_prizes<C>(&mut self, prizes: C)
    where
        C: IntoIterator<Item = Prize>,
    {
        self.prizes.clear();
        self.prizes = prizes.into_iter().collect();
    }

    /// Returns the entrants.
    pub fn entrants(&self) -> std::collections::hash_map::Values<'_, K, U> {
        self.entrants.values()
    }

    /// Returns the entrants, which are mutable.
    pub fn entrants_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, K, U> {
        self.entrants.values_mut()
    }

    pub fn shuffle<'myself>(&'myself mut self, rng: &mut impl Rng)
    where
        'myself: 'u,
    {
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
            self.shuffle_tree(rng);
        } else {
            self.shuffle_array(rng);
        }
    }

    pub fn shuffle_tree<'myself>(&'myself mut self, rng: &mut impl Rng)
    where
        'myself: 'u,
    {
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
    pub fn shuffle_array<'myself, R>(&'myself mut self, rng: &mut R)
    where
        R: Rng,
        'myself: 'u,
    {
        use std::iter::repeat_n;
        // Shuffle twice would cause double spend.
        if self.shuffled {
            return;
        }

        // Shuffle the slots, each entrant has `entrant.ticket_count()` slots.
        // Use the entrant's key to point back to itself.
        // The complexity shuffling would be both O(n).
        let tickets_god_only_knows_which_entrant = {
            use rand::seq::SliceRandom;
            let mut ret = Vec::from_iter(
                self.entrants
                    .values()
                    // FIXME:
                    // I suspect insisting keys to be `Clone` be more straightforward and cheaper
                    // than triggering (potentially expensive) hash algorithm each time...?
                    // BUT
                    // there may be some hash algorithms that are expensive to calculate
                    // and expensive to clone...
                    .flat_map(|entrant| {
                        // HACK: calculate hash each time to circumvent lifetime of keys of the map
                        repeat_n((), entrant.ticket_count()).map(|_| entrant.key())
                    }),
            );
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
        for prize in prizes {
            for ticket in tickets.by_ref() {
                // It is possible to use raw pointer to reduce both key production and hashing costs.
                // However, it requires unsafe.
                // Fortunately, this is not recursive, and relative simple to check the boundary.
                if self.entrants.get_mut(&ticket).unwrap().add_prize(prize) {
                    break;
                }
            }
        }
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
