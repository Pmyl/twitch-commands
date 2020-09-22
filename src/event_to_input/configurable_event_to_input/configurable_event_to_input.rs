use async_trait::async_trait;
use crate::event_to_input::event_to_input::EventToInput;
use crate::stream_interface::events::{ChatEvents};
use crate::utils::run_on_stream::StreamItemReceiver;
use crate::system_input::system_input::{SystemInput};
use crate::system_input::enigo::enigo_system_input::EnigoSystemInput;

pub struct ConfigurableEventToInput {
    controller: EnigoSystemInput,
    configuration: Configuration,
}

pub struct Configuration {
    options: Vec<(String, String)>
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            options: Vec::new()
        }
    }
}

// impl ConfigurableEventToInput {
//     pub fn new(configuration: Configuration) -> ConfigurableEventToInput {
//         ConfigurableEventToInput { controller: EnigoSystemInput::new(), configuration }
//     }
// }

#[async_trait]
impl EventToInput for ConfigurableEventToInput {
    async fn execute(&mut self, event: ChatEvents) {
        event_to_input(event, &self.configuration, &mut self.controller).await;
    }
}

#[async_trait]
impl StreamItemReceiver for ConfigurableEventToInput {
    type Item = ChatEvents;
    async fn receive(&mut self, event: ChatEvents) {
        self.execute(event).await;
    }
}

async fn event_to_input(event: ChatEvents, config: &Configuration, system_input: &mut impl SystemInput) {
    let ChatEvents::Message(message) = event;
    let option = config.options.iter()
        .find(|&opt| { opt.0 == message.content });

    if let Some((_, action)) = option {
        if action == "up" {
            system_input.arrow_up();
        } else if action == "down" {
            system_input.arrow_down();
        } else if action == "up_down" {
            system_input.arrow_up();
            system_input.arrow_down();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use crate::stream_interface::events::ChatMessage;
    use crate::{at, mock_system_input};

    mock_system_input!();
    macro_rules! assert_do_nothing {
        ($fn_name:ident, $message:expr, $config_options:expr) => {
            #[test]
            fn $fn_name() {
                let mut mock = MockSystemInput::new();

                at!(event_to_input(
                    message_event($message.to_string()),
                    &Configuration {
                        options: $config_options
                    },
                    &mut mock
                ));
            }
        }
    }

    macro_rules! assert_actions {
        ($fn_name:ident, $action_name:expr, $($expected_method:ident)+) => {
            #[test]
            fn $fn_name() {
                let mut mock = MockSystemInput::new();

                $(
                mock.$expected_method()
                    .once()
                    .return_const(());
                )+

                at!(event_to_input(
                    message_event("a message".to_string()),
                    &Configuration {
                        options: vec![("a message".to_string(), $action_name.to_string())]
                    },
                    &mut mock
                ));
            }
        }
    }

    assert_do_nothing!(empty_event_empty_config_do_nothing, "", vec![]);
    assert_do_nothing!(event_says_up_config_not_match_do_nothing, "I said up", vec![("".to_string(), "".to_string())]);
    assert_do_nothing!(empty_message_config_for_up_do_nothing, "", vec![("I said up".to_string(), "up".to_string())]);
    assert_do_nothing!(event_match_config_for_unhandled_action_do_nothing, "A", vec![("A".to_string(), "unhandled".to_string())]);

    assert_actions!(event_match_config_for_press_up_then_press_up, "up", expect_arrow_up);
    assert_actions!(event_match_config_for_press_down_then_press_down, "down", expect_arrow_down);
    assert_actions!(event_match_config_for_press_updown_then_press_up_then_down, "up_down", expect_arrow_up expect_arrow_down);

    fn message_event(content: String) -> ChatEvents {
        ChatEvents::Message(ChatMessage { name: "".to_string(), content, is_mod: false })
    }
}
