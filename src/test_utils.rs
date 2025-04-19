use crate::prize::Prize;
use crate::user::User;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct UserWithLog<'p> {
    uuid: usize,
    tickets_count: usize,
    prizes: Vec<&'p Prize>,
    log: Rc<RefCell<Vec<&'p Prize>>>,
}

impl<'p> User<'p> for UserWithLog<'p> {
    type Key = usize;
    fn key(&self) -> Self::Key {
        self.uuid
    }
    fn add_prize(&mut self, prize: &'p crate::prize::Prize) -> bool {
        if <Self as User>::ticket_count(self) > 0 {
            self.prizes.push(prize);
            self.log.borrow_mut().push(prize);
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

impl<'p> UserWithLog<'p> {
    pub(crate) fn new(uuid: usize) -> (Self, Rc<RefCell<Vec<&'p Prize>>>) {
        let external_log = Rc::new(RefCell::new(vec![]));
        (
            Self {
                uuid,
                tickets_count: uuid,
                prizes: vec![],
                log: Rc::clone(&external_log),
            },
            external_log,
        )
    }
}
