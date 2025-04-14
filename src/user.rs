use crate::prize::Prize;
use uuid::Uuid;

pub trait User<'a, K> {
    /// returns the uuid of the user
    fn key(&self) -> K;
    /// total count of lottery ticket
    fn ticket_count(&self) -> usize;
    /// add the prize, return false if failed
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

impl<'a> SinglePrizeUser<'a> {
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

    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> User<'a, Uuid> for SinglePrizeUser<'a> {
    fn key(&self) -> Uuid {
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

impl<'a> MultiPrizeUser<'a> {
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

    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> User<'a, Uuid> for MultiPrizeUser<'a> {
    fn key(&self) -> Uuid {
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
