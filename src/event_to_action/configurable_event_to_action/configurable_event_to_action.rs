use crate::event_to_action::event_to_action::{EventToAction};
use crate::stream_interface::events::{ChatEvent};
use crate::utils::run_on_stream::StreamItemReceiver;
use crate::actions::action::{Action, ActionCategory};
use crate::utils::app_config::{Mapping, MappingConfig};

pub struct ConfigurableEventToAction {
    configuration: Configuration
}

pub struct Configuration {
    pub message_options: Vec<ConfigOption>,
    pub event_options: Vec<ConfigOption>
}

pub struct ConfigOption {
    pub id: String,
    pub actions: ActionCategory
}

impl From<Mapping> for Configuration {
    fn from(mapping: Mapping) -> Self {
        Configuration {
            message_options: mapping.config.iter()
                .filter(|c| c.source == "message")
                .map(|message_action| into_option(message_action))
                .collect::<Vec<ConfigOption>>(),

            event_options: mapping.config.iter()
                .filter(|c| c.source == "event")
                .map(|message_action| into_option(message_action))
                .collect::<Vec<ConfigOption>>()
        }
    }
}

fn into_option(mapping: &MappingConfig) -> ConfigOption {
    ConfigOption {
        id: mapping.id.clone(),
        actions: condense_actions(mapping.actions.clone(), mapping.category.clone())
    }
}

fn condense_actions(actions: Vec<String>, category: String) -> ActionCategory {
    let action_sequence = actions.iter()
        .map(|action_baby| action_birth(action_baby))
        .collect::<Vec<Action>>();

    let condensed_action: Action;

    if action_sequence.is_empty() {
        panic!("At least one action is required, found 0.");
    } else if action_sequence.len() == 1 {
        condensed_action = action_sequence[0].clone();
    } else {
        condensed_action = Action::Sequence(action_sequence);
    }

    if category.is_empty() {
        ActionCategory::Uncategorized(condensed_action)
    } else {
        ActionCategory::WithCategory(category.clone(), condensed_action)
    }
}

fn action_birth(action_to_map: &str) -> Action {
    match action_to_map {
        keydown if keydown.starts_with("kd") => Action::KeyRawDown(keydown.replace("kd", "").parse::<u16>().unwrap()),
        keyup if keyup.starts_with("ku") => Action::KeyRawUp(keyup.replace("ku", "").parse::<u16>().unwrap()),
        wait if wait.starts_with("w") => Action::WaitFor(wait.replace("w", "").parse::<u64>().unwrap()),
        atomic_sequence if atomic_sequence.starts_with("~") =>
            Action::AtomicSequence(atomic_sequence.split("~").skip(1).map(|matryoshka_baby| action_birth(matryoshka_baby)).collect()),
        wrong_action_description => panic!("Provided wrong action description {}", wrong_action_description)
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            message_options: Vec::new(),
            event_options: Vec::new()
        }
    }
}

impl ConfigurableEventToAction {
    pub fn new(configuration: Configuration) -> ConfigurableEventToAction {
        ConfigurableEventToAction { configuration }
    }
}

impl EventToAction for ConfigurableEventToAction {
    fn execute(&mut self, event: ChatEvent) -> Option<ActionCategory> {
        event_to_action(event, &self.configuration)
    }

    fn custom_categories(&mut self) -> Vec<String> {
        vec![String::from("1")]
    }
}

impl StreamItemReceiver for ConfigurableEventToAction {
    type Item = ChatEvent;
    type Output = Option<ActionCategory>;
    fn receive(&mut self, event: ChatEvent) -> Option<ActionCategory> {
        self.execute(event)
    }
}

fn event_to_action(event: ChatEvent, config: &Configuration) -> Option<ActionCategory> {
    let option;

    match event {
        ChatEvent::Message(message) => {
            option = config.message_options.iter()
                .find(|&opt| opt.id == message.content)?;
        }
    }

    Some(option.actions.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream_interface::events::ChatMessage;

    impl Configuration {
        fn messages(message_options: Vec<ConfigOption>) -> Self {
            Configuration { message_options, event_options: vec![] }
        }
    }

    macro_rules! assert_return_nothing {
        ($fn_name:ident, $message:expr, $config:expr) => {
            #[test] fn $fn_name() {
                assert!(event_to_action(
                    message_event(s!($message)),
                    &$config
                ).is_none());
            }
        }
    }

    macro_rules! assert_actions {
        ($fn_name:ident, action $actions:expr, category $category:literal, returns $expected_action:expr) => {
            #[test] fn $fn_name() {
                let maybe_generated = event_to_action(
                    message_event(s!("a message")),
                    &Mapping { config: vec![MappingConfig { id: s!("a message"), actions: $actions, category: s!($category), source: s!("message") } ] }.into()
                );

                assert!(maybe_generated.is_some());
                let generated = maybe_generated.unwrap();
                assert!(generated == $expected_action);
            }
        };
        ($fn_name:ident, action $actions:expr, returns $expected_action:expr) => {
            assert_actions!($fn_name, action $actions, category "", returns $expected_action);
        }
    }

    macro_rules! s {
        ($str:literal) => {
            $str.to_string()
        }
    }

    assert_return_nothing!(empty_event_empty_config_return_nothing, "", Configuration::default());
    assert_return_nothing!(event_says_up_config_not_match_return_nothing, "I said up",
        Configuration::messages(vec![ConfigOption { id: s!(""), actions: ActionCategory::Uncategorized(Action::WaitFor(1)) }])
    );
    assert_return_nothing!(empty_message_config_for_up_return_nothing, "",
        Configuration::messages(vec![ConfigOption { id: s!("I said up"), actions: ActionCategory::Uncategorized(Action::WaitFor(1)) }])
    );

    assert_actions!(event_match_config_for_kd_number_then_key_down_raw_40,
     action     vec![s!("kd40")],
     returns    ActionCategory::Uncategorized(Action::KeyRawDown(40)));

    assert_actions!(event_match_config_for_kd_number_then_key_down_raw_100,
     action     vec![s!("kd100")],
     returns    ActionCategory::Uncategorized(Action::KeyRawDown(100)));

    assert_actions!(event_match_config_for_ku_number_then_key_up_raw_100,
     action     vec![s!("ku100")],
     returns    ActionCategory::Uncategorized(Action::KeyRawUp(100)));

    assert_actions!(event_match_config_for_multiple_kd_number_then_sequence_key_down_raw,
     action     vec![s!("kd100"), s!("kd35")],
     returns    ActionCategory::Uncategorized(Action::Sequence(vec![Action::KeyRawDown(100), Action::KeyRawDown(35)])));

    assert_actions!(event_match_config_for_w_number_then_wait,
     action     vec![s!("w1500")],
     returns    ActionCategory::Uncategorized(Action::WaitFor(1500)));

    assert_actions!(event_match_config_for_empty_category_then_uncategorized,
     action     vec![s!("w1500")],
     category   "",
     returns    ActionCategory::Uncategorized(Action::WaitFor(1500)));

    assert_actions!(event_match_config_for_category_then_action_categorized,
     action     vec![s!("w1500")],
     category   "a",
     returns    ActionCategory::WithCategory(s!("a"), Action::WaitFor(1500)));

    assert_actions!(event_match_config_for_tilde_then_action_atomic_sequence,
     action     vec![s!("~w1500~ku10~w1500")],
     returns    ActionCategory::Uncategorized(Action::AtomicSequence(vec![Action::WaitFor(1500), Action::KeyRawUp(10), Action::WaitFor(1500)])));

    fn message_event(content: String) -> ChatEvent {
        ChatEvent::Message(ChatMessage { name: "".to_string(), content, is_mod: false })
    }
}
