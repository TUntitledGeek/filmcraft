//! Bounded idle buffer reuse. A pending asynchronous operation owns its lease;
//! the same storage cannot be handed to another operation before that lease drops.
use std::{cell::RefCell, rc::Rc};

pub struct Pool<T> {
    idle: RefCell<Option<(u32, T)>>,
    maximum: u32,
}
pub struct Lease<T> {
    pool: Rc<Pool<T>>,
    buffer: Option<(u32, T)>,
}

impl<T> Pool<T> {
    pub fn new(maximum: u32) -> Rc<Self> {
        Rc::new(Self { idle: RefCell::new(None), maximum })
    }

    pub fn acquire<E>(self: &Rc<Self>, minimum: u32, allocate: impl FnOnce(u32) -> Result<T, E>) -> Result<Lease<T>, E> {
        let idle = self.idle.try_borrow_mut().ok().and_then(|mut v| v.take());
        let buffer = match idle.filter(|(capacity, _)| *capacity >= minimum) {
            Some(buffer) => buffer,
            None => (minimum, allocate(minimum)?),
        };
        Ok(Lease { pool: self.clone(), buffer: Some(buffer) })
    }
}

impl<T> Lease<T> {
    pub fn value(&self) -> Option<&T> {
        self.buffer.as_ref().map(|(_, v)| v)
    }
}
impl<T> Drop for Lease<T> {
    fn drop(&mut self) {
        if let Some((capacity, value)) = self.buffer.take()
            && capacity <= self.pool.maximum
            && let Ok(mut idle) = self.pool.idle.try_borrow_mut()
            && idle.as_ref().is_none_or(|(old, _)| *old < capacity)
        {
            *idle = Some((capacity, value));
        }
    }
}
