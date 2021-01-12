use crate::actions::action::{Action, ActionContainer, ActionLocker};
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
    pub fn run(&mut self, actions: &mut Vec<ActionContainer>) {
        if actions.is_empty() {
            return;
        }
        let mut action_container = actions.remove(0);
        debug!("Check action type {:?}", action_container.action);

        match action_container.action {
            Action::Sequence(mut vector) => {
                let action_in_sequence = vector.remove(0);
                action_container.action = action_in_sequence;
                actions.insert(0, action_container.clone());
                if vector.len() > 0 {
                    let mut new_container = action_container;
                    new_container.action = Action::Sequence(vector);
                    actions.insert(1, new_container);
                };
            },
            Action::AtomicSequence(vector) => {
                for action in vector.iter() {
                    self.execute(action)
                }
            },
            Action::WaitFor(ms) => {
                action_container.action = Action::WaitUntil(Instant::now().add(Duration::from_millis(ms)));
                actions.insert(0, action_container);
            },
            Action::WaitUntil(until) => {
                if until > Instant::now() {
                    actions.insert(0, action_container);
                }
            },
            executable_action => self.execute(&executable_action)
        };
    }

    pub fn can_handle(&mut self, actions: &mut Vec<ActionContainer>) -> bool {
        if let Some(action_container) = actions.first() {
            return match action_container.pause_on {
                ActionLocker::None => true,
                ActionLocker::MousePressed => !self.input_system.is_mouse_left_down()
            }
        }

        true
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
