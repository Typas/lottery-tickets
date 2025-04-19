use crate::prize::Prize;
use crate::user::User;

#[derive(Debug)]
pub struct SimpleUser<'u> {
    uuid: usize,
    tickets_count: usize,
    prizes: Vec<&'u Prize>,
}

impl<'u> User<'u> for SimpleUser<'u> {
    type Key = usize;
    fn key(&self) -> Self::Key {
        self.uuid
    }
    fn add_prize(&mut self, prize: &'u crate::prize::Prize) -> bool {
        if <Self as User>::ticket_count(self) > 0 {
            self.prizes.push(prize);
            true
        } else {
            false
        }
    }
    fn ticket_count(&self) -> usize {
        self.tickets_count
    }
    fn has_prize(&self) -> bool {
        !self.prizes.is_empty()
    }
}

impl<'u> SimpleUser<'u> {
    pub(crate) fn new(uuid: usize) -> Self {
        Self {
            uuid,
            tickets_count: uuid,
            prizes: vec![],
        }
    }
}
