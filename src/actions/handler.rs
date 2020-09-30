use crate::actions::action::Action;
use crate::system_input::system_input::SystemInput;
use crate::system_input::custom_system_input::custom_system_input::CustomSystemInput;
use std::time::{Instant, Duration};
use std::ops::Add;

pub struct ActionHandler {
    input_system: CustomSystemInput
}

impl Default for ActionHandler {
    fn default() -> Self {
        ActionHandler { input_system: CustomSystemInput::new() }
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

    pub fn can_handle(&mut self) -> bool {
        !self.input_system.is_mouse_left_down()
    }

    fn execute(&mut self, action: &Action) {
        debug!("Executing {:?}", action);
        match action {
            Action::KeyRawDown(raw) => self.input_system.key_down(*raw),
            Action::KeyRawUp(raw) => self.input_system.key_up(*raw),
            Action::MoveMouseOf(x, y) => self.input_system.move_mouse_of(*x, *y),
            non_executable_action => error!("Found wrong action nesting, example AtomicSequence with Sequence as an action {:?}", non_executable_action)
        }
    }
}
