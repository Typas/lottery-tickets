use crate::prize::Prize;
use crate::user::User;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct GenericUser<'p> {
    uuid: usize,
    tickets_count: usize,
    prizes: Vec<&'p Prize>,
    log: Rc<RefCell<Vec<&'p Prize>>>,
}

impl<'p> User<'p> for GenericUser<'p> {
    type Key = usize;
    fn key(&self) -> Self::Key {
        self.uuid
    }
    fn add_prize(&mut self, prize: &'p crate::prize::Prize) -> bool {
        if self.tickets_count > 0 {
            self.prizes.push(prize);
            self.tickets_count -= 1;
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

impl<'p> GenericUser<'p> {
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

    pub(crate) fn with_tickets_count(uuid: usize, tickets_count: usize) -> (Self, Rc<RefCell<Vec<&'p Prize>>>) {
        let external_log = Rc::new(RefCell::new(vec![]));
        (
            Self {
                uuid,
                tickets_count,
                prizes: vec![],
                log: Rc::clone(&external_log),
            },
            external_log,
        )
    }
}

#[derive(Debug)]
pub(crate) struct CapacityOneUser<'p> {
    uuid: usize,
    tickets_count: usize,
    pub prize: Option<&'p Prize>,
    log: Rc<RefCell<Vec<&'p Prize>>>,
}

impl<'p> User<'p> for CapacityOneUser<'p> {
    type Key = usize;
    fn key(&self) -> Self::Key {
        self.uuid
    }
    fn add_prize(&mut self, prize: &'p crate::prize::Prize) -> bool {
        if self.tickets_count > 0 && self.prize.is_none() {
            self.prize = Some(prize);
            self.tickets_count -= 1;
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
        self.prize.is_some()
    }
}

impl<'p> CapacityOneUser<'p> {
    pub(crate) fn new(uuid: usize) -> (Self, Rc<RefCell<Vec<&'p Prize>>>) {
        let external_log = Rc::new(RefCell::new(vec![]));
        (
            Self {
                uuid,
                tickets_count: uuid,
                prize: None,
                log: Rc::clone(&external_log),
            },
            external_log,
        )
    }

    pub(crate) fn with_tickets_count(uuid: usize, tickets_count: usize) -> (Self, Rc<RefCell<Vec<&'p Prize>>>) {
        let external_log = Rc::new(RefCell::new(vec![]));
        (
            Self {
                uuid,
                tickets_count,
                prize: None,
                log: Rc::clone(&external_log),
            },
            external_log,
        )
    }
}
