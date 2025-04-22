use crate::prize::Prize;
use crate::entrant::Entrant;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct GenericEntrant<'p> {
    uuid: usize,
    tickets_count: usize,
    prizes: Vec<&'p Prize>,
    log: Rc<RefCell<Vec<&'p Prize>>>,
}

impl<'p> Entrant<'p> for GenericEntrant<'p> {
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

impl<'p> GenericEntrant<'p> {
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

    #[allow(unused)]
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
pub(crate) struct CapacityOneEntrant<'p> {
    uuid: usize,
    tickets_count: usize,
    pub prize: Option<&'p Prize>,
    log: Rc<RefCell<Vec<&'p Prize>>>,
}

impl<'p> Entrant<'p> for CapacityOneEntrant<'p> {
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

impl<'p> CapacityOneEntrant<'p> {
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
