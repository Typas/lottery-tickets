use std::hash::Hash;

use crate::prize::Prize;
use uuid::Uuid;

pub trait User<'a> {
    type Key: Hash + Eq;
    /// returns the uuid of the user
    fn key(&self) -> Self::Key;
    /// total count of lottery ticket
    fn ticket_count(&self) -> usize;
    /// add the prize, return false if failed
    ///
    /// Assumed to be monotone, in the sense that once returned `false`,
    /// it's never `true` again,
    /// effectively the implementation cannot be picky about the prizes
    fn add_prize(&mut self, prize: &'a Prize) -> bool;
    /// check if the user has at least one prize
    fn has_prize(&self) -> bool;
}

pub struct SinglePrizeUser<'a> {
    count: usize, // total count of lottery ticket
    name: String,
    id: Uuid,
    prize: Option<&'a Prize>,
}

pub struct MultiPrizeUser<'a> {
    count: usize, // total count of lottery ticket
    name: String,
    id: Uuid,
    prizes: Vec<&'a Prize>,
}

pub struct UserBuilder {
    count: usize, // total count of lottery ticket
    name: Option<String>,
    id: Option<Uuid>,
}

impl SinglePrizeUser<'_> {
    pub fn new(id: Uuid, name: String, ticket_count: usize) -> Self {
        Self {
            count: ticket_count,
            name,
            id,
            prize: None,
        }
    }

    pub fn prize(&self) -> Option<&Prize> {
        self.prize
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> User<'a> for SinglePrizeUser<'a> {
    type Key = Uuid;

    fn key(&self) -> Self::Key {
        self.id
    }

    fn ticket_count(&self) -> usize {
        self.count
    }

    fn add_prize(&mut self, prize: &'a Prize) -> bool {
        if self.has_prize() {
            false
        } else {
            self.prize = Some(prize);
            true
        }
    }

    fn has_prize(&self) -> bool {
        self.prize.is_some()
    }
}

impl MultiPrizeUser<'_> {
    pub fn new(id: Uuid, name: String, ticket_count: usize) -> Self {
        Self {
            count: ticket_count,
            name,
            id,
            prizes: Vec::new(),
        }
    }

    pub fn prizes(&self) -> &Vec<&Prize> {
        &self.prizes
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> User<'a> for MultiPrizeUser<'a> {
    type Key = Uuid;
    fn key(&self) -> Self::Key {
        self.id
    }

    fn ticket_count(&self) -> usize {
        self.count
    }

    fn add_prize(&mut self, prize: &'a Prize) -> bool {
        self.prizes.push(prize);
        true
    }

    fn has_prize(&self) -> bool {
        !self.prizes.is_empty()
    }
}

impl Default for UserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl UserBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            id: None,
            count: 0,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    pub fn ticket_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build_single<'a>(self) -> SinglePrizeUser<'a> {
        SinglePrizeUser {
            count: self.count,
            name: self.name.unwrap_or_default(),
            id: self.id.unwrap_or_else(Uuid::new_v4),
            prize: None,
        }
    }

    pub fn build_multiple<'a>(self) -> MultiPrizeUser<'a> {
        MultiPrizeUser {
            count: self.count,
            name: self.name.unwrap_or_default(),
            id: self.id.unwrap_or_else(Uuid::new_v4),
            prizes: Vec::new(),
        }
    }
}
