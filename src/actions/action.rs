#[derive(Debug)]
#[derive(Clone)]
pub enum Action {
    // Categorized(Box<(u16, Action)>),
    KeyRawDown(u16),
    KeyRawUp(u16),
    Wait(u64),
    Sequence(Vec<Action>),
    Parallel(Vec<Action>)
}
