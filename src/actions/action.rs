use std::time::Instant;

#[derive(Debug)]
#[derive(Clone)]
pub enum Action {
    KeyRawDown(u16),
    KeyRawUp(u16),
    WaitFor(u64),
    WaitUntil(Instant),
    Sequence(Vec<Action>),
    AtomicSequence(Vec<Action>)
}

pub enum ActionCategory {
    WithCategory(String, Action),
    Uncategorized(Action)
}
