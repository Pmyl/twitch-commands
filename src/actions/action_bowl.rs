use crate::actions::action::Action;

pub struct ActionBowl {
    actions: Vec<Action>
}

impl ActionBowl {
    pub fn new() -> Self {
        ActionBowl { actions: Vec::new() }
    }

    pub fn insert(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub fn pick(&mut self) -> Option<Action> {
        if self.actions.is_empty() {
            return None;
        }
        Some(self.actions.remove(0))
    }
}
