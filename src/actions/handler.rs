use crate::actions::action::Action;
use crate::system_input::enigo::enigo_system_input::EnigoSystemInput;
use crate::system_input::system_input::SystemInput;

pub struct ActionHandler {
    pub input_system: EnigoSystemInput,
}

impl Default for ActionHandler {
    fn default() -> Self {
        ActionHandler { input_system: EnigoSystemInput::new() }
    }
}

impl ActionHandler {
    pub async fn feed(&mut self, action: Action) {
        let mut next_action: Option<Action> = Some(action);

        while let Some(current_action) = next_action {
            println!("Check action type {:?}", current_action);
            next_action = None;

            match current_action {
                Action::Sequence(mut vector) => {
                    let action_in_sequence = vector.pop().unwrap();
                    next_action = if vector.len() > 0 {
                        Some(Action::Sequence(vector))
                    } else {
                        None
                    };
                    self.execute(&action_in_sequence).await
                },
                Action::Parallel(vector) => {
                    for action in vector.iter() {
                        self.execute_sync(action)
                    }
                },
                executable_action => self.execute(&executable_action).await
            };
        }
    }

    async fn execute(&mut self, action: &Action) {
        println!("Executing all {:?}", action);
        match action {
            Action::Wait(ms) => self.input_system.delay_for(*ms).await,
            maybe_sync => { println!("Async not found"); self.execute_sync(maybe_sync) }
        }
    }

    fn execute_sync(&mut self, action: &Action) {
        println!("Executing sync {:?}", action);
        match action {
            Action::KeyRawDown(raw) => self.input_system.key_down(*raw),
            Action::KeyRawUp(raw) => self.input_system.key_down(*raw),
            non_sync_action => eprintln!("HOW IS IT POSSIBLE??? execute_sync RUN WITH A NON SYNC ACTION {:?}", non_sync_action)
        }
    }
}
