use crate::prize::Prize;
use uuid::Uuid;

pub trait User<'a> {
    fn name(&self) -> &str;
    fn id(&self) -> Uuid;
    fn set_begin(&mut self, begin: usize);
    fn ticket_count(&self) -> usize;
    fn indexes(&self) -> (usize, usize);
    // add the prize, return false if failed
    fn add_prize(&mut self, prize: &'a Prize) -> bool;
    fn has_prize(&self) -> bool;
}

pub struct SinglePrizeUser<'a> {
    begin: usize, // the starting index of `numbers` in `Tickets`
    count: usize, // `end` would be C++ style, exclusive one
    name: String,
    id: Uuid,
    prize: Option<&'a Prize>,
}

pub struct MultiPrizeUser<'a> {
    begin: usize, // the starting index of `numbers` in `Tickets`
    count: usize, // `end` would be C++ style, exclusive one
    name: String,
    id: Uuid,
    prizes: Vec<&'a Prize>,
}

pub struct UserBuilder {
    name: Option<String>,
    id: Option<Uuid>,
}

impl<'a> SinglePrizeUser<'a> {
    pub fn new(id: Uuid, name: String) -> Self {
        Self {
            begin: 0,
            count: 0,
            name,
            id,
            prize: None,
        }
    }

    pub fn prize(&self) -> Option<&Prize> {
        self.prize
    }
}

impl<'a> User<'a> for SinglePrizeUser<'a> {
    fn set_begin(&mut self, start: usize) {
        self.begin = start;
    }

    fn ticket_count(&self) -> usize {
        self.count
    }

    fn indexes(&self) -> (usize, usize) {
        (self.begin, self.begin + self.count)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> Uuid {
        self.id
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
    pub fn new(id: Uuid, name: String) -> Self {
        Self {
            begin: 0,
            count: 0,
            name,
            id,
            prizes: Vec::new(),
        }
    }

    pub fn prizes(&self) -> &Vec<&Prize> {
        &self.prizes
    }
}

impl<'a> User<'a> for MultiPrizeUser<'a> {
    fn set_begin(&mut self, start: usize) {
        self.begin = start;
    }

    fn ticket_count(&self) -> usize {
        self.count
    }

    fn indexes(&self) -> (usize, usize) {
        (self.begin, self.begin + self.count)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> Uuid {
        self.id
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

    pub fn build_single<'a>(self) -> SinglePrizeUser<'a> {
        SinglePrizeUser {
            begin: 0,
            count: 0,
            name: self.name.unwrap_or_default(),
            id: self.id.unwrap_or_else(Uuid::new_v4),
            prize: None,
        }
    }

    pub fn build_multiple<'a>(self) -> MultiPrizeUser<'a> {
        MultiPrizeUser {
            begin: 0,
            count: 0,
            name: self.name.unwrap_or_default(),
            id: self.id.unwrap_or_else(Uuid::new_v4),
            prizes: Vec::new(),
        }
    }
}
