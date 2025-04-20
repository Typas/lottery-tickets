use std::{
    collections::HashMap,
    hash::{Hash, RandomState},
    iter::repeat_n,
    marker::PhantomData,
};

use rand::Rng;

use crate::{prize::Prize, space_efficient_shuffler, user::User};
pub struct Tickets<'user, K, U, S = RandomState>
where
    K: Hash + Eq,
    U: User<'user, Key = K>,
{
    /// Determine whether the lottery has been shuffled and done.
    shuffled: bool,
    /// The users in a hash map, use .users() to get the result
    users: HashMap<K, U, S>,
    /// The prizes, the lower the index, the higher the priority.
    prizes: Vec<Prize>,
    /// The lifetime is actually refering to those `impl User` in the map,
    /// which in turn is referring to `Prize` in this exact struct (`Self::prizes`)
    ///
    /// Rust complains if not explicitly used in any of the fields,
    /// thus the marker.
    _marker: PhantomData<&'user U>,
}

impl<'u, K, U> Default for Tickets<'u, K, U>
where
    K: Hash + Eq,
    U: User<'u, Key = K>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'u, K, U> Tickets<'u, K, U>
where
    K: Hash + Eq,
    U: User<'u, Key = K>,
{
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            prizes: Vec::new(),
            shuffled: false,
            _marker: PhantomData,
        }
    }

    pub fn with_user_capacity(cap: usize) -> Self {
        Self {
            users: HashMap::with_capacity(cap),
            prizes: Vec::new(),
            shuffled: false,
            _marker: PhantomData,
        }
    }
}

impl<'u, K, U, S> Tickets<'u, K, U, S>
where
    K: Hash + Eq,
    U: User<'u, Key = K>,
    S: std::hash::BuildHasher + std::default::Default,
{
    pub fn with_user_capacity_and_hasher(cap: usize, hasher: S) -> Self {
        Self {
            users: HashMap::with_capacity_and_hasher(cap, hasher),
            prizes: Vec::new(),
            shuffled: false,
            _marker: PhantomData,
        }
    }

    pub fn with_hasher(hasher: S) -> Self {
        Self {
            users: HashMap::with_hasher(hasher),
            prizes: Vec::new(),
            shuffled: false,
            _marker: PhantomData,
        }
    }

    /// Check if the prize has been distributed.
    pub fn shuffled(&self) -> bool {
        self.shuffled
    }

    /// Add a user to the lottery.
    /// When the keys collide, it would return the old user.
    pub fn add_user(&mut self, user: U) -> Option<U> {
        self.users.insert(user.key(), user)
    }

    /// Set all the users in the lottery.
    /// It is possible to have less user if the keys collide.
    pub fn set_users<'a, C>(&mut self, users: C)
    where
        C: IntoIterator<Item = U>,
    {
        self.users.clear();
        self.users = users.into_iter().map(|u| (u.key(), u)).collect();
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

    /// Returns the users.
    pub fn users(&self) -> std::collections::hash_map::Values<'_, K, U> {
        self.users.values()
    }

    /// Returns the users, which are mutable.
    pub fn users_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, K, U> {
        self.users.values_mut()
    }

    pub fn shuffle<'myself>(&'myself mut self, rng: &mut impl Rng)
    where
        'myself: 'u,
    {
        // FIXME: this is just a guess.
        // Let `u` be the number of users, `t` be the number of tickets, `p` be the number of prizes.
        // Assuming the size of "tree" would require 24 * 2 * u * log(u),
        // while the size of "array" would require 16 * t.
        // We might compare `3*u*log(u)` with `t`.
        let array_est: usize = self.users.values().map(|u| u.ticket_count()).sum();
        let tree_est = 3 * self.users.len() * self.users.len().ilog2() as usize;
        if array_est <= tree_est {
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
            space_efficient_shuffler::SpaceEfficientShuffler::new(self.users.values_mut());
        while space_efficient_shuffler.try_draw_one(rng, &mut prizes) {}
    }

    /// Shuffle the slots and distribute the prizes to the users.
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

        // Shuffle the slots, each user has `user.ticket_count()` slots.
        // Use the user's key to point back to itself.
        // The complexity shuffling would be both O(n).
        let tickets_god_only_knows_which_user = {
            use rand::seq::SliceRandom;
            let mut ret = Vec::from_iter(
                self.users
                    .values()
                    // FIXME:
                    // I suspect insisting keys to be `Clone` be more straightforward and cheaper
                    // than triggering (potentially expensive) hash algorithm each time...?
                    // BUT
                    // there may be some hash algorithms that are expensive to calculate
                    // and expensive to clone...
                    .flat_map(|user| {
                        // HACK: calculate hash each time to circumvent lifetime of keys of the map
                        repeat_n((), user.ticket_count()).map(|_| user.key())
                    }),
            );
            ret.shuffle(rng);
            ret
        };

        // Map the prize to user, the overall time complexity would be O(k)
        // once the key-mapped user has been saturated, try next user
        // For example, let's call `this.prizes[i]` `pi`, `this.users.keys()[i]` `ki`.
        // `p0.count = 1`, `p1.count = 2`, `p2.count = 3`.
        // We will have `prizes = [&p0, &p1, &p1, &p2, &p2, &p2]`
        // and the tickets would be like `[k5, k4, k4, k3, k2, k5, k6, k7...]`.
        // over iteration the mapping would be
        // p0 -> u5
        // p1 -> u4
        // p1 -> u3 // skip u4, assuming each user would only hold 1 prize
        // p2 -> u2
        // p2 -> u6 // skip u5, same assumption
        // p2 -> u7
        let prizes = self.prizes.iter().flat_map(|p| repeat_n(p, p.count()));
        let mut tickets = tickets_god_only_knows_which_user.into_iter();
        for prize in prizes {
            for ticket in tickets.by_ref() {
                // It is possible to use raw pointer to reduce both key production and hashing costs.
                // However, it requires unsafe.
                // Fortunately, this is not recursive, and relative simple to check the boundary.
                if self.users.get_mut(&ticket).unwrap().add_prize(prize) {
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
    use std::collections::BTreeSet;

    use super::Tickets;
    use crate::prize::PrizeBuilder;
    use crate::space_efficient_shuffler::SpaceEfficientShuffler;
    use crate::test_utils::CapacityOneUser;
    use crate::test_utils::GenericUser;

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
    fn test_space_efficient_shuffler() {
        let mut rng = rand::rng();
        const MAX_PRIZE_COUNT: usize = 100;
        const NUM_USERS: usize = 65536;
        let (prizes, num_prizes) = {
            let mut n = 0;
            let ret = Vec::from_iter((0..MAX_PRIZE_COUNT).map(|x| {
                n += x;
                PrizeBuilder::new().count(x).name(format!("{x}")).build()
            }));
            (ret, n)
        };
        let (users, log) = (0..NUM_USERS).map(GenericUser::new).collect::<(Vec<_>, Vec<_>)>();
        let mut tickets = Tickets::new();
        prizes.into_iter().for_each(|p| tickets.add_prize(p));
        users.into_iter().for_each(|u| {
            tickets.add_user(u);
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
