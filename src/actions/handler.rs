use crate::actions::action::Action;
use crate::system_input::enigo::enigo_system_input::EnigoSystemInput;
use crate::system_input::system_input::SystemInput;
use std::time::{Instant, Duration};
use std::ops::Add;

pub struct ActionHandler {
    input_system: EnigoSystemInput
}

impl Default for ActionHandler {
    fn default() -> Self {
        ActionHandler { input_system: EnigoSystemInput::new() }
    }
}

impl ActionHandler {
    pub fn run(&mut self, actions: &mut Vec<Action>) {
        if actions.is_empty() {
            return;
        }
        let action = actions.remove(0);
        debug!("Check action type {:?}", action);

        match action {
            Action::Sequence(mut vector) => {
                let action_in_sequence = vector.remove(0);
                actions.insert(0, action_in_sequence);
                if vector.len() > 0 {
                    actions.insert(1, Action::Sequence(vector));
                };
            },
            Action::AtomicSequence(vector) => {
                for action in vector.iter() {
                    self.execute(action)
                }
            },
            Action::WaitFor(ms) => {
                actions.insert(0, Action::WaitUntil(Instant::now().add(Duration::from_millis(ms))));
            },
            Action::WaitUntil(until) => {
                if until > Instant::now() {
                    actions.insert(0, action);
                }
            },
            executable_action => self.execute(&executable_action)
        };
    }

    fn execute(&mut self, action: &Action) {
        debug!("Executing {:?}", action);
        match action {
            Action::KeyRawDown(raw) => self.input_system.key_down(*raw),
            Action::KeyRawUp(raw) => self.input_system.key_down(*raw),
            non_executable_action => error!("Found wrong action nesting, example AtomicSequence with Sequence as an action {:?}", non_executable_action)
        }
    }
}
