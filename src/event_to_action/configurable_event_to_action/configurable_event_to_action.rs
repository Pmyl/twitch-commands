use std::collections::HashSet;
use crate::event_to_action::event_to_action::{EventToAction};
use crate::stream_interface::events::{ChatEvent};
use crate::utils::run_on_stream::StreamItemReceiver;
use crate::actions::action::{Action, ActionCategory};
use crate::utils::app_config::{Mapping, MappingConfig};
use std::num::ParseIntError;
use derivative::{Derivative};

pub struct ConfigurableEventToAction {
    configuration: Configuration
}

pub struct Configuration {
    pub message_options: Vec<ConfigOption>,
    pub action_options: Vec<ConfigActionOption>
}

pub trait ConfigOptionWithActions {
    fn get_actions(&self) -> ActionCategory;
    fn get_times(&self) -> Option<u16>;
    fn set_times(&mut self, times: u16);
}

pub trait ConfigOptionWithActionsTrait {
    fn consume_actions(&mut self) -> ActionCategory;
    fn can_be_executed(&self) -> bool;
}

impl<T: ConfigOptionWithActions> ConfigOptionWithActionsTrait for T {
    fn consume_actions(&mut self) -> ActionCategory {
        match self.get_times() {
            None => self.get_actions(),
            Some(limit) => {
                if limit > 0 {
                    self.set_times(limit - 1);
                } else {
                    error!("Actions consumed even if it finished the limit, something wrong with the code!");
                }

                self.get_actions()
            }
        }
    }

    fn can_be_executed(&self) -> bool {
        self.get_times().is_none() || self.get_times().unwrap() > 0
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ConfigOption {
    pub id: String,
    #[derivative(Debug="ignore")]
    pub actions: ActionCategory,
    pub times_limit: Option<u16>
}

impl ConfigOptionWithActions for ConfigOption {
    fn get_actions(&self) -> ActionCategory {
        self.actions.clone()
    }
    
    fn get_times(&self) -> Option<u16> {
        self.times_limit
    }
    
    fn set_times(&mut self, times: u16) {
        self.times_limit = Some(times);
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ConfigActionOption {
    pub id: String,
    #[derivative(Debug="ignore")]
    pub actions: ActionCategory,
    #[derivative(Debug="ignore")]
    pub comparison: Box<dyn Fn(String) -> bool>,
    pub action_name: String,
    pub times_limit: Option<u16>
}

impl ConfigOptionWithActions for ConfigActionOption {
    fn get_actions(&self) -> ActionCategory {
        self.actions.clone()
    }

    fn get_times(&self) -> Option<u16> {
        self.times_limit
    }

    fn set_times(&mut self, times: u16) {
        self.times_limit = Some(times);
    }
}

impl From<Mapping> for Configuration {
    fn from(mapping: Mapping) -> Self {
        Configuration {
            message_options: mapping.config.iter()
                .filter(|c| c.source == "message")
                .map(|message_action| into_option(message_action))
                .collect::<Vec<ConfigOption>>(),

            action_options: mapping.config.iter()
                .filter(|c| c.source == "action")
                .map(|message_action| into_action_option(message_action))
                .collect::<Vec<ConfigActionOption>>()
        }
    }
}

fn into_option(mapping: &MappingConfig) -> ConfigOption {
    ConfigOption {
        id: mapping.id.clone(),
        actions: condense_actions(mapping.actions.clone(), mapping.category.clone()),
        times_limit: mapping.limit
    }
}

fn into_action_option(mapping: &MappingConfig) -> ConfigActionOption {
    ConfigActionOption {
        id: mapping.id.clone(),
        actions: condense_actions(mapping.actions.clone(), mapping.category.clone()),
        comparison: into_comparison_fn(mapping.comparison.clone(), mapping.id.clone()),
        action_name: mapping.name.clone(),
        times_limit: mapping.limit
    }
}

fn into_comparison_fn(comparison_type: String, id: String) -> Box<dyn Fn(String) -> bool> {
    match comparison_type.as_str() {
        "range" => comparison_range_builder(id),
        _ => Box::new(move |s: String| s == id)
    }
}

fn comparison_range_builder(range_config: String) -> Box<dyn Fn(String) -> bool> {
    let ranges = range_config.split("-").collect::<Vec<&str>>().into_iter().map(|s| s.parse::<u64>()).collect::<Vec<Result<u64, ParseIntError>>>();

    if let [Ok(low_bound), Ok(up_bound)] = ranges[..] {
        Box::new(move |s: String| comparison_range(s, low_bound, up_bound))
    } else {
        panic!("Range comparison failed, range config is not properly defined: {}", range_config);
    }
}

fn comparison_range(input: String, low_bound: u64, up_bound: u64) -> bool {
    if let Ok(input_number) = input.parse::<u64>() {
        low_bound <= input_number && up_bound >= input_number
    } else {
        error!("Range comparison failed, input is not a number: {}", input);
        false
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
        mouse_relative if mouse_relative.starts_with("mr") => {
            let coordinates = mouse_relative
                .replace("mr", "")
                .split("x")
                .map(|xy| xy.parse::<i32>().unwrap())
                .collect::<Vec<i32>>();
            Action::MoveMouseOf(coordinates[0], coordinates[1])
        },
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
            action_options: Vec::new()
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
        event_to_action(event, &mut self.configuration)
    }

    fn custom_categories(&mut self) -> Vec<String> {
        let mut categories_map: HashSet<String> = HashSet::new();

        for option in &self.configuration.message_options {
            match option.actions {
                ActionCategory::WithCategory(ref category_name, _) => { categories_map.insert(category_name.clone()); () },
                _ => ()
            }
        }

        for option in &self.configuration.action_options {
            match option.actions {
                ActionCategory::WithCategory(ref category_name, _) => { categories_map.insert(category_name.clone()); () },
                _ => ()
            }
        }

        categories_map.into_iter().collect::<Vec<String>>()
    }
}

impl StreamItemReceiver for ConfigurableEventToAction {
    type Item = ChatEvent;
    type Output = Option<ActionCategory>;
    fn receive(&mut self, event: ChatEvent) -> Option<ActionCategory> {
        self.execute(event)
    }
}

fn event_to_action(event: ChatEvent, config: &mut Configuration) -> Option<ActionCategory> {
    let actions;

    match event.clone() {
        ChatEvent::Message(message) => {
            let option = config.message_options.iter_mut()
                .filter(|opt| opt.can_be_executed())
                .find(|opt| opt.id == message.content)?;
            actions = option.consume_actions();
            info!("Executing action {:?} from event {:?}", option, event);
        },
        ChatEvent::Action(action) => {
            let option = config.action_options.iter_mut()
                .filter(|opt| opt.can_be_executed())
                .find(|opt| action.action_name == opt.action_name && (opt.comparison)(action.action_id.clone()))?;
            actions = option.consume_actions();
            info!("Executing action {:?} from event {:?}", option, event);
        },
    }

    Some(actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream_interface::events::ChatMessage;
    use crate::{s};

    impl Configuration {
        fn messages(message_options: Vec<ConfigOption>) -> Self {
            Configuration { message_options, action_options: vec![] }
        }
    }

    macro_rules! assert_return_nothing {
        ($fn_name:ident, $message:expr, $config:expr) => {
            #[test] fn $fn_name() {
                assert!(event_to_action(
                    message_event(s!($message)),
                    &mut $config
                ).is_none());
            }
        }
    }

    macro_rules! assert_actions {
        ($fn_name:ident, action $actions:expr, category $category:literal, returns $expected_action:expr) => {
            #[test] fn $fn_name() {
                let maybe_generated = event_to_action(
                    message_event(s!("a message")),
                    &mut Mapping { config: vec![MappingConfig { id: s!("a message"), actions: $actions, category: s!($category), source: s!("message"), comparison: s!(""), name: s!(""), limit: None } ] }.into()
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

    assert_return_nothing!(empty_event_empty_config_return_nothing, "", Configuration::default());
    assert_return_nothing!(event_says_up_config_not_match_return_nothing, "I said up",
        Configuration::messages(vec![ConfigOption { id: s!(""), actions: ActionCategory::Uncategorized(Action::WaitFor(1)), times_limit: None }])
    );
    assert_return_nothing!(empty_message_config_for_up_return_nothing, "",
        Configuration::messages(vec![ConfigOption { id: s!("I said up"), actions: ActionCategory::Uncategorized(Action::WaitFor(1)), times_limit: None }])
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

    assert_actions!(event_match_config_for_mr_coordinates_then_move_mouse,
     action     vec![s!("mr100x110")],
     returns    ActionCategory::Uncategorized(Action::MoveMouseOf(100, 110)));

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

    #[test]
    fn configuration_created_without_categories_return_no_custom_categories() {
        let mut event_to_action = ConfigurableEventToAction {
            configuration: Configuration {
                message_options: vec![ConfigOption { actions: ActionCategory::Uncategorized(Action::KeyRawUp(1)), id: s!(""), times_limit: None }],
                action_options: vec![ConfigActionOption { actions: ActionCategory::Uncategorized(Action::KeyRawUp(2)), id: s!(""), action_name: s!(""), comparison: Box::new(|_: String| false), times_limit: None }]
            }
        };

        assert_eq!(event_to_action.custom_categories().len(), 0);
    }

    #[test]
    fn configuration_created_wit_categories_return_list_of_custom_categories() {
        let mut event_to_action = ConfigurableEventToAction {
            configuration: Configuration {
                message_options: vec![
                    ConfigOption { actions: ActionCategory::Uncategorized(Action::KeyRawUp(1)), id: s!(""), times_limit: None },
                    ConfigOption { actions: ActionCategory::WithCategory(s!("1"), Action::KeyRawUp(1)), id: s!(""), times_limit: None }
                ],
                action_options: vec![
                    ConfigActionOption { actions: ActionCategory::WithCategory(s!("custom_text"), Action::KeyRawUp(2)), id: s!(""), action_name: s!(""), comparison: Box::new(|_: String| false), times_limit: None },
                    ConfigActionOption { actions: ActionCategory::Uncategorized(Action::KeyRawUp(2)), id: s!(""), action_name: s!(""), comparison: Box::new(|_: String| false), times_limit: None }
                ]
            }
        };

        assert_eq!(event_to_action.custom_categories().len(), 2);
        assert!(event_to_action.custom_categories().contains(&s!("1")));
        assert!(event_to_action.custom_categories().contains(&s!("custom_text")));
    }

    fn message_event(content: String) -> ChatEvent {
        ChatEvent::Message(ChatMessage { name: s!(""), content, is_mod: false })
    }
}
