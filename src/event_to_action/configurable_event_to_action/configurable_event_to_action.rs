use crate::event_to_action::event_to_action::{EventToAction};
use crate::stream_interface::events::{ChatEvent};
use crate::utils::run_on_stream::StreamItemReceiver;
use crate::system_input::system_input::{SystemInput};
use crate::system_input::enigo::enigo_system_input::EnigoSystemInput;
use crate::actions::action::{Action, ActionCategory};

pub struct ConfigurableEventToAction {
    controller: EnigoSystemInput,
    configuration: Configuration
}

pub struct Configuration {
    pub options: Vec<(String, String)>
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            options: Vec::new()
        }
    }
}

impl ConfigurableEventToAction {
    pub fn new(configuration: Configuration) -> ConfigurableEventToAction {
        ConfigurableEventToAction { controller: EnigoSystemInput::new(), configuration }
    }
}

impl EventToAction for ConfigurableEventToAction {
    fn execute(&mut self, event: ChatEvent) -> Option<ActionCategory> {
        event_to_action(event, &self.configuration, &mut self.controller)
    }

    fn custom_categories(&mut self) -> Vec<String> {
        vec![String::from("1")]
    }
}

impl StreamItemReceiver for ConfigurableEventToAction {
    type Item = ChatEvent;
    type Output = Option<ActionCategory>;
    fn receive(&mut self, event: ChatEvent) -> Option<ActionCategory> {
        let maybe_action = self.execute(event);
        match maybe_action {
            Some(action) => {
                Some(action)
            },
            _ => None
        }
    }
}

fn event_to_action(event: ChatEvent, config: &Configuration, _system_input: &mut impl SystemInput) -> Option<ActionCategory> {
    let ChatEvent::Message(message) = event;
    let option = config.options.iter()
        .find(|&opt| { opt.0 == message.content });

    match option {
        Some((_, action)) => {
            match action.as_ref() {
                "up" => Some(ActionCategory::WithCategory(String::from("1"), Action::KeyRawDown(38))),
                "down" => Some(ActionCategory::Uncategorized(Action::KeyRawDown(40))),
                "up_down" => Some(ActionCategory::Uncategorized(Action::Sequence(vec![Action::KeyRawDown(38), Action::WaitFor(1000), Action::KeyRawDown(40)]))),
                "find" => Some(ActionCategory::Uncategorized(Action::Sequence(vec![Action::KeyRawDown(17), Action::KeyRawDown(70), Action::KeyRawUp(17)]))),
                "find_atomic" => Some(ActionCategory::Uncategorized(Action::AtomicSequence(vec![Action::KeyRawDown(17), Action::KeyRawDown(70), Action::KeyRawUp(17)]))),
                _ => None
            }
        },
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use tokio::time::{delay_until, Instant};
    use crate::stream_interface::events::ChatMessage;
    use crate::{mock_system_input};

    mock_system_input!();

    macro_rules! assert_do_nothing {
        ($fn_name:ident, $message:expr, $config_options:expr) => {
            #[test] fn $fn_name() {
                event_to_action(
                    message_event($message.to_string()),
                    &Configuration { options: $config_options },
                    &mut MockSystemInput::new()
                );
            }
        }
    }

    macro_rules! assert_actions {
        ($fn_name:ident, action $action_name:expr, calls $($expected_method:ident $times:literal times)+ $(, and wait $($ms:literal ms $wait_times:literal times)+)?) => {
            #[test] fn $fn_name() {
                let mut mock = MockSystemInput::new();

                $(
                mock.$expected_method()
                    .times($times)
                    .return_const(());
                )+

                $(
                    $(
                    mock.expect_delay_for()
                        .with(eq($ms))
                        .times($wait_times)
                        .returning(|_ms| delay_until(Instant::now()));
                    )+
                )?

                event_to_action(
                    message_event("a message".to_string()),
                    &Configuration {
                        options: vec![("a message".to_string(), $action_name.to_string())]
                    },
                    &mut mock
                );
            }
        }
    }

    assert_do_nothing!(empty_event_empty_config_do_nothing, "", vec![]);
    assert_do_nothing!(event_says_up_config_not_match_do_nothing, "I said up", vec![("".to_string(), "".to_string())]);
    assert_do_nothing!(empty_message_config_for_up_do_nothing, "", vec![("I said up".to_string(), "up".to_string())]);
    assert_do_nothing!(event_match_config_for_unhandled_action_do_nothing, "A", vec![("A".to_string(), "unhandled".to_string())]);

    assert_actions!(event_match_config_for_press_up_then_press_up,
     action "up",
     calls  expect_arrow_up 1 times);

    assert_actions!(event_match_config_for_press_down_then_press_down,
     action "down",
     calls  expect_arrow_down 1 times);

    assert_actions!(event_match_config_for_press_updown_then_press_up_then_down,
     action "up_down",
     calls  expect_arrow_up 1 times
            expect_arrow_down 1 times,
     and wait 1000 ms 1 times);

    fn message_event(content: String) -> ChatEvent {
        ChatEvent::Message(ChatMessage { name: "".to_string(), content, is_mod: false })
    }
}
