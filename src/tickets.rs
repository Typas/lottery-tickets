use std::{collections::{BinaryHeap, HashMap}, hash::RandomState};

use rand::Rng;
use uuid::Uuid;

use crate::{prize::Prize, user::User};
pub struct Tickets<U, S = RandomState>
where
    for<'a> U: User<'a>,
{
    users: HashMap<Uuid, U, S>,
    prizes: Vec<Prize>,
}

impl<U> Tickets<U>
where
    for<'a> U: User<'a>,
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

impl<U, S> Tickets<U, S>
where
    S: std::hash::BuildHasher + std::default::Default,
    for<'a> U: User<'a>,
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
        self.users.insert(user.id(), user)
    }

    pub fn set_users<'a, C>(&mut self, users: C)
    where
        C: IntoIterator<Item = U>,
    {
        self.users.clear();
        self.users = users.into_iter().map(|u| (u.id(), u)).collect();
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
        self.users_mut().fold(total_count, |c, u| {
            u.set_begin(c);
            c + u.ticket_count()
        });
        let range = 0..total_count;
        let mut numbers: Vec<usize> = range.clone().collect();

        // shuffle
        for i in range.clone() {
            let j = rng.random_range(range.clone());
            numbers.swap(i, j);
        }

        // dispatch prizes
        let prizes = &self.prizes; // thank you stack borrow
        let mut heap = BinaryHeap::new();
        for user in self.users.values_mut() {
            let (begin, end) = user.indexes();
            for i in begin..end {
                if let Some(p) = Self::check_prize(prizes, numbers[i]) {
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
        todo!("shuffle several times to ensure every user would accept only zero or one prize.");
    }

    pub fn users(&self) -> std::collections::hash_map::Values<'_, Uuid, U> {
        self.users.values()
    }

    pub fn users_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, Uuid, U> {
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

impl<'a> Eq for PriorityPrize<'a> {
}
