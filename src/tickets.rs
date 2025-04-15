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
    users: HashMap<K, U, S>,
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

    pub fn shuffle<R>(&mut self, rng: &mut R)
    where
        R: Rng,
    {
        let total_count = 0;
        // begin indexes corresponding to users
        let mut user_begins: Vec<usize> = Vec::with_capacity(self.users.len());
        self.users.values().fold(total_count, |c, u| {
            user_begins.push(c);
            c + u.ticket_count()
        });

        let god_only_knows = {
            // unique random number for each ticket of each user
            use rand::seq::SliceRandom;
            let mut ret = Vec::from_iter(0..total_count);
            ret.shuffle(rng);
            ret
        };

        // dispatch prizes
        let prizes = &self.prizes; // thank you stack borrow
        let mut heap = BinaryHeap::new();
        for (user, begin) in self.users.values_mut().zip(user_begins) {
            let end = begin + user.ticket_count();
            for i in begin..end {
                if let Some(p) = Self::check_prize(prizes, god_only_knows[i]) {
                    heap.push(PriorityPrize::new(p, &prizes[p]));
                }
            }
            // heap guaranteed the priority, but introduces extra O(k) complexity
            while let Some(pp) = heap.pop() {
                user.add_prize(pp.reference);
            }
        }
    }

    pub fn one_prize_per_user_shuffle<R>(&mut self, rng: &mut R)
    where
        R: Rng,
    {
        // "priority" here is actually the count
        let mut copied_prizes: Vec<_> = self
            .prizes
            .iter()
            .map(|p| PriorityPrize::new(p.count(), p))
            .collect();
        // begin indexes corresponding to users
        let mut prize_count: usize = copied_prizes.iter().map(|pz| pz.priority).sum();

        // Shuffle multiple times, only dispatch best prize to the user.
        // Once the user has prize, isolate it from the lottery.
        // The worst case of shuffling would be O(n),
        // and each shuffling would take O(n) time.
        // Therefore, this results in a total of O(n^2) worse case.
        while prize_count != 0 {
            let mut num_tickets_partial_sum: Vec<usize> =
                Vec::with_capacity(self.users.values().skip_while(|u| u.has_prize()).count());
            let num_tickets = self
                .users
                .values()
                .skip_while(|u| u.has_prize())
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
            let prizes = &self.prizes; // thank you stack borrow
            let mut max_prize_idx: Option<usize> = None;
            for (user, num_tickets_partial_sum_till_user) in self
                .users
                .values_mut()
                .skip_while(|u| u.has_prize())
                .zip(num_tickets_partial_sum)
            {
                let num_tickets_partial_sum_till_after_user =
                    num_tickets_partial_sum_till_user + user.ticket_count();
                for i in num_tickets_partial_sum_till_user..num_tickets_partial_sum_till_after_user
                {
                    if let Some(prize_idx) =
                        Self::check_copied_prize(&copied_prizes, god_only_knows[i])
                    {
                        max_prize_idx =
                            Some(max_prize_idx.map_or(prize_idx, |mp| mp.min(prize_idx)));
                    }
                }

                if let Some(idx) = max_prize_idx {
                    user.add_prize(&prizes[idx]);
                    copied_prizes[idx].priority -= 1;
                }
            }

            // calcuate the remaining prizes
            prize_count = copied_prizes.iter().map(|p| p.priority).sum();
        }
    }

    pub fn users(&self) -> std::collections::hash_map::Values<'_, K, U> {
        self.users.values()
    }

    pub fn users_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, K, U> {
        self.users.values_mut()
    }

    fn check_prize(prizes: &[Prize], mut n: usize) -> Option<usize> {
        for (i, prize) in prizes.iter().enumerate() {
            if n < prize.count() {
                return Some(i);
            }
            n -= prize.count();
        }
        None
    }

    fn check_copied_prize(prizes: &[PriorityPrize], mut n: usize) -> Option<usize> {
        for (i, prize) in prizes.iter().enumerate() {
            if n < prize.priority {
                return Some(i);
            }
            n -= prize.priority;
        }
        None
    }
}

struct PriorityPrize<'a> {
    priority: usize,
    reference: &'a Prize,
}

impl<'a> PriorityPrize<'a> {
    fn new(priority: usize, reference: &'a Prize) -> Self {
        Self {
            priority,
            reference,
        }
    }
}

impl<'a> Ord for PriorityPrize<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl<'a> PartialOrd for PriorityPrize<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<'a> PartialEq for PriorityPrize<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.priority.eq(&other.priority)
    }
}

impl<'a> Eq for PriorityPrize<'a> {}
