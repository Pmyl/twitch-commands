use std::time::Instant;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum Action {
    KeyRawDown(u16),
    KeyRawUp(u16),
    MoveMouseOf(i32, i32),
    WaitFor(u64),
    WaitUntil(Instant),
    Sequence(Vec<Action>),
    AtomicSequence(Vec<Action>)
}

#[derive(Clone)]
#[derive(PartialEq)]
pub enum ActionCategory {
    WithCategory(String, Action),
    Uncategorized(Action)
}
