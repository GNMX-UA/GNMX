use std::rc::Rc;
use std::cell::RefCell;

pub type Shared<T> = Rc<RefCell<T>>;