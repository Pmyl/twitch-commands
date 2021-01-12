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

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct ActionContainer {
    pub action: Action,
    pub pause_on: ActionLocker
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum ActionLocker {
    None,
    MousePressed
}

#[derive(Clone)]
#[derive(PartialEq)]
pub enum ActionCategory {
    WithCategory(String, ActionContainer),
    Uncategorized(ActionContainer)
}
