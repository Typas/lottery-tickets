use std::{
    collections::HashMap,
    hash::{Hash, RandomState},
};

use rand::Rng;

use crate::{prize::Prize, user::User};
pub struct Tickets<K, U, S = RandomState>
where
    K: Hash + Eq,
    for<'a> U: User<'a, Key = K>,
{
    shuffled: bool,
    // The users in a hash map, use .users() to get the result
    users: HashMap<K, U, S>,
    // The prizes, the lower the index, the higher the priority.
    prizes: Vec<Prize>,
}

impl<K, U> Tickets<K, U>
where
    K: Hash + Eq,
    for<'a> U: User<'a, Key = K>,
{
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            prizes: Vec::new(),
            shuffled: false,
        }
    }

    pub fn with_user_capacity(cap: usize) -> Self {
        Self {
            users: HashMap::with_capacity(cap),
            prizes: Vec::new(),
            shuffled: false,
        }
    }
}

impl<K, U, S> Tickets<K, U, S>
where
    K: Hash + Eq,
    for<'a> U: User<'a, Key = K>,
    S: std::hash::BuildHasher + std::default::Default,
{
    pub fn with_user_capacity_and_hasher(cap: usize, hasher: S) -> Self {
        Self {
            users: HashMap::with_capacity_and_hasher(cap, hasher),
            prizes: Vec::new(),
            shuffled: false,
        }
    }

    pub fn with_hasher(hasher: S) -> Self {
        Self {
            users: HashMap::with_hasher(hasher),
            prizes: Vec::new(),
            shuffled: false,
        }
    }

    /// Check if the prize has been distributed.
    pub fn shuffled(&self) -> bool {
        self.shuffled
    }

    /// Add a user to the lottery.
    /// When the keys collide, it would return the old user.
    pub fn add_user<'a>(&mut self, user: U) -> Option<U> {
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

    /// Shuffle the slots and distribute the prizes to the users.
    pub fn shuffle<R>(&mut self, rng: &mut R)
    where
        R: Rng,
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
        let mut prizes = self.prizes.iter().flat_map(|p| repeat_n(p, p.count()));
        let mut tickets = tickets_god_only_knows_which_user.into_iter();
        while let Some(prize) = prizes.next() {
            while let Some(ticket) = tickets.next() {
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
