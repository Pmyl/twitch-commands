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
        match action.as_ref() {
            "up" => system_input.arrow_up(),
            "down" => system_input.arrow_down(),
            "up_down" => {
                system_input.arrow_up();
                system_input.delay_for(1000).await;
                system_input.arrow_down();
            },
            _ => ()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use tokio::time::{delay_until, Instant};
    use crate::stream_interface::events::ChatMessage;
    use crate::{at, mock_system_input};

    mock_system_input!();

    macro_rules! assert_do_nothing {
        ($fn_name:ident, $message:expr, $config_options:expr) => {
            #[test] fn $fn_name() {
                at!(event_to_input(
                    message_event($message.to_string()),
                    &Configuration { options: $config_options },
                    &mut MockSystemInput::new()
                ));
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

    fn message_event(content: String) -> ChatEvents {
        ChatEvents::Message(ChatMessage { name: "".to_string(), content, is_mod: false })
    }
}
