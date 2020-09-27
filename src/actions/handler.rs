use crate::actions::action::Action;
use crate::system_input::enigo::enigo_system_input::EnigoSystemInput;
use crate::system_input::system_input::SystemInput;

pub struct ActionHandler {
    input_system: EnigoSystemInput,
    actions: Vec<Action>
}

impl Default for ActionHandler {
    fn default() -> Self {
        ActionHandler { input_system: EnigoSystemInput::new(), actions: Vec::<Action>::new() }
    }
}

impl ActionHandler {
    pub fn feed(&mut self, action: Action) {
        self.actions.push(action);
        println!("Fed, now {:?}", self.actions);
    }

    pub async fn run(&mut self) {
        if self.actions.is_empty() {
            return;
        }

        let mut next_action: Option<Action> = Some(self.actions.remove(0));

        while let Some(current_action) = next_action {
            println!("Check action type {:?}", current_action);
            next_action = None;

            match current_action {
                Action::Sequence(mut vector) => {
                    let action_in_sequence = vector.remove(0);
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
        println!("Executing async version of {:?}", action);
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
