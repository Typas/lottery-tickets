use std::hash::Hash;

use crate::prize::Prize;
use uuid::Uuid;

pub trait Entrant<'a> {
    type Key: Hash + Eq;
    /// returns the uuid of the entrant
    fn key(&self) -> Self::Key;
    /// total count of lottery ticket
    fn ticket_count(&self) -> usize;
    /// add the prize, return false if failed
    ///
    /// Assumed to be monotone, in the sense that once returned `false`,
    /// it's never `true` again,
    /// effectively the implementation cannot be picky about the prizes
    fn add_prize(&mut self, prize: &'a Prize) -> bool;
    /// check if the entrant has at least one prize
    fn has_prize(&self) -> bool;
}

pub struct SinglePrizeEntrant<'a> {
    count: usize, // total count of lottery ticket
    name: String,
    id: Uuid,
    prize: Option<&'a Prize>,
}

pub struct MultiPrizeEntrant<'a> {
    count: usize, // total count of lottery ticket
    name: String,
    id: Uuid,
    prizes: Vec<&'a Prize>,
}

pub struct EntrantBuilder {
    count: usize, // total count of lottery ticket
    name: Option<String>,
    id: Option<Uuid>,
}

impl SinglePrizeEntrant<'_> {
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

impl<'a> Entrant<'a> for SinglePrizeEntrant<'a> {
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

impl MultiPrizeEntrant<'_> {
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

impl<'a> Entrant<'a> for MultiPrizeEntrant<'a> {
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

impl Default for EntrantBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EntrantBuilder {
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

    pub fn build_single<'a>(self) -> SinglePrizeEntrant<'a> {
        SinglePrizeEntrant {
            count: self.count,
            name: self.name.unwrap_or_default(),
            id: self.id.unwrap_or_else(Uuid::new_v4),
            prize: None,
        }
    }

    pub fn build_multiple<'a>(self) -> MultiPrizeEntrant<'a> {
        MultiPrizeEntrant {
            count: self.count,
            name: self.name.unwrap_or_default(),
            id: self.id.unwrap_or_else(Uuid::new_v4),
            prizes: Vec::new(),
        }
    }
}
