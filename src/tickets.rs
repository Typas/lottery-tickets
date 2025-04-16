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
        use std::cell::RefCell;
        use std::iter::repeat_n;
        use std::rc::Rc;

        // Shuffle multiple times, only dispatch best prize to the user.
        // Once the user has prize, isolate it from the lottery.
        // The worst case of shuffling would be O(n),
        // and each shuffling would take O(n) time.
        // Therefore, this results in a total of O(n^2) worse case.
        let tickets_god_only_knows_which_user = {
            use rand::seq::SliceRandom;
            let mut ret = Vec::from_iter(
                self.users
                    .values_mut()
                    .map(RefCell::new)
                    .map(Rc::new)
                    .flat_map(|rc| repeat_n(rc.clone(), rc.borrow().ticket_count())),
            );
            ret.shuffle(rng);
            ret
        };
        let mut prizes = self.prizes.iter().flat_map(|p| repeat_n(p, p.count()));
        let mut tickets = tickets_god_only_knows_which_user.into_iter();
        while let Some(prize) = prizes.next() {
            while let Some(ticket) = tickets.next() {
                // `Rc<RefCell<T>>` FTW
                //
                // Pros:
                // easy mode lifetime around `&mut`
                //
                // Cons:
                // 1. may prove troublesome if concurrent version were required
                // 2. runtime overhead
                if ticket.borrow_mut().add_prize(prize) {
                    break;
                }
            }
        }
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
        Self { priority, count }
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
