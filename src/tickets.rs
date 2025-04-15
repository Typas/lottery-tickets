use std::{
    collections::{BinaryHeap, HashMap},
    hash::{Hash, RandomState},
};

use rand::Rng;

use crate::{prize::Prize, user::User};
pub struct Tickets<K, U, S = RandomState>
where
    K: Hash + Eq,
    for<'a> U: User<'a, Key = K>,
{
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
        }
    }

    pub fn with_user_capacity(cap: usize) -> Self {
        Self {
            users: HashMap::with_capacity(cap),
            prizes: Vec::new(),
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
        }
    }

    pub fn with_hasher(hasher: S) -> Self {
        Self {
            users: HashMap::with_hasher(hasher),
            prizes: Vec::new(),
        }
    }

    pub fn add_user<'a>(&mut self, user: U) -> Option<U> {
        self.users.insert(user.key(), user)
    }

    pub fn set_users<'a, C>(&mut self, users: C)
    where
        C: IntoIterator<Item = U>,
    {
        self.users.clear();
        self.users = users.into_iter().map(|u| (u.key(), u)).collect();
    }

    pub fn add_prize(&mut self, prize: Prize) {
        self.prizes.push(prize);
    }

    pub fn set_prizes<C>(&mut self, prizes: C)
    where
        C: IntoIterator<Item = Prize>,
    {
        self.prizes.clear();
        self.prizes = prizes.into_iter().collect();
    }

    pub fn users(&self) -> std::collections::hash_map::Values<'_, K, U> {
        self.users.values()
    }

    pub fn users_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, K, U> {
        self.users.values_mut()
    }

    pub fn shuffle<R>(&mut self, rng: &mut R)
    where
        R: Rng,
    {
        let mut prize_counts: Vec<_> = self.prizes.iter().map(|p| p.count()).collect();

        // Shuffle multiple times, only dispatch best prize to the user.
        // Once the user has prize, isolate it from the lottery.
        // The worst case of shuffling would be O(n),
        // and each shuffling would take O(n) time.
        // Therefore, this results in a total of O(n^2) worse case.
        while prize_counts.iter().any(|pc| *pc > 0) {
            let mut num_tickets_partial_sum: Vec<usize> =
                Vec::with_capacity(self.users.values().filter(|u| !u.has_prize()).count());
            let num_tickets = self
                .users
                .values()
                .filter(|u| !u.has_prize())
                .fold(0, |c, u| {
                    num_tickets_partial_sum.push(c);
                    c + u.ticket_count()
                });

            // unique random number for each ticket of each user
            let god_only_knows = {
                use rand::seq::SliceRandom;
                let mut ret = Vec::from_iter(0..num_tickets);
                ret.shuffle(rng);
                ret
            };

            // dispatch prizes
            for (user, num_tickets_partial_sum_till_user) in self
                .users
                .values_mut()
                .filter(|u| !u.has_prize())
                .zip(num_tickets_partial_sum)
            {
                // extra O(k) to ensure the user would always get the largest ones
                let mut heap = BinaryHeap::new();
                let num_tickets_partial_sum_till_after_user =
                    num_tickets_partial_sum_till_user + user.ticket_count();
                for i in num_tickets_partial_sum_till_user..num_tickets_partial_sum_till_after_user
                {
                    if let Some(prize_idx) =
                        Self::check_prize(&prize_counts, god_only_knows[i])
                    {
                        heap.push(prize_idx);
                    }
                }

                // release extra prizes
                while let Some(idx) = heap.pop() {
                    if !user.add_prize(&self.prizes[idx]) {
                        break;
                    }
                    prize_counts[idx] -= 1;
                }
            }
        }
    }

    fn check_prize(prize_counts: &[usize], mut n: usize) -> Option<usize> {
        for (i, prize_count) in prize_counts.iter().enumerate() {
            if n < *prize_count {
                return Some(i);
            }
            n -= *prize_count;
        }
        None
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct PrizeCounter {
    priority: usize,
    count: usize,
}

#[allow(dead_code)]
impl PrizeCounter {
    fn new(priority: usize, count: usize) -> Self {
        Self {
            priority,
            count,
        }
    }
}

impl Ord for PrizeCounter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for PrizeCounter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl PartialEq for PrizeCounter {
    fn eq(&self, other: &Self) -> bool {
        self.priority.eq(&other.priority)
    }
}

impl Eq for PrizeCounter {}
