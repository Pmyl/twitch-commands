use twitchchat::{Connector, Dispatcher, Runner};
use futures::stream::{Stream};
use tokio::stream::StreamExt;
use std::fmt::{Display, Formatter, Error};
use std::sync::Arc;
use twitchchat::messages::{Privmsg};
use twitch_api2::pubsub::channel_points::{ChannelPointsChannelV1, ChannelPointsChannelV1Reply, Redemption, Reward};
use twitch_api2::pubsub::{TopicSubscribe, Response, TopicData, TwitchResponse};
use websocket::{ClientBuilder, Message};
use websocket::message::OwnedMessage::{Text, Binary, Close};
use tokio::sync::mpsc::{channel};
use twitch_api2::types::User;
use std::borrow::Borrow;
use curl::easy::{Easy, List};
use serde::{Deserialize};
use crate::stream_interface::events::{ChatEvent, ChatMessage, ChatAction};
use crate::{s};
use crate::utils::app_config::TwitchStreamConfig;

pub async fn connect_to_twitch(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    info!("Connecting to stream: {}", options);
    let chat_stream = create_messages_stream(options.clone()).await;
    let channel_rewards_stream = create_channel_rewards_stream(options).await;

    chat_stream.merge(channel_rewards_stream)
}

async fn create_channel_rewards_stream(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    let topic = ChannelPointsChannelV1 {
        channel_id: user_id_from_login_name(options.clone())
    };

    let command = TopicSubscribe::listen(
        &[topic],
        options.token,
        s!("????")
    ).to_command().expect("Serializing failed");

    info!("Subscribing to channel rewards");
    let mut client = ClientBuilder::new("wss://pubsub-edge.twitch.tv").unwrap().connect(None).unwrap();
    client.send_message(&Message::text(command)).unwrap();

    let (mut tx, rx) = channel::<ChatEvent>(100);

    tokio::spawn(async move {
        for message in client.incoming_messages() {
            match message {
                Ok(msg) => {
                    match msg {
                        Text(text) => {
                            let result = Response::parse(&text).unwrap();
                            match result {
                                Response::Message {
                                    data: TopicData::ChannelPointsChannelV1 {
                                        reply, ..
                                    }
                                } => {
                                    if let ChannelPointsChannelV1Reply::RewardRedeemed {
                                        redemption: Redemption {
                                            reward: Reward {
                                                title, ..
                                            },
                                            user: User {
                                                display_name: user_name, ..
                                            }, ..
                                        }, ..
                                    } = reply.borrow() {
                                        info!("Redeemed {:?}!", title);
                                        if let Err(send_error) = tx.send(
                                            ChatEvent::Action(ChatAction {
                                                name: user_name.to_owned(),
                                                action_id: "reward_redeem:".to_owned() + title
                                            })
                                        ).await {
                                            error!("Error sending event {:?}", send_error);
                                        };
                                    }
                                },
                                Response::Response(tr) => {
                                    if let TwitchResponse { error: Some(ref error), .. } = tr {
                                        if !error.eq("") {
                                            panic!("Connection to channel rewards failed {}", error);
                                        }
                                    }
                                    debug!("PubSub message response parsed, unhandled twitch response {:?}", tr)
                                },
                                Response::Reconnect => debug!("PubSub message response parsed, unhandled reconnect"),
                                Response::Message { .. } => debug!("PubSub message response parsed, it's a message but not the one we want"),
                                _ => ()
                            }
                        },
                        Binary(b) => debug!("PubSub unhandled binary message received: {:?}", b),
                        Close(b) => debug!("PubSub unhandled close message received: {:?}", b),
                        _ => ()
                    }
                },
                Err(e) => debug!("error when getting incoming message? {:?}", e)
            }
        }
    });
    info!("Subscribed!");

    rx
}

#[derive(Deserialize)]
struct TwitchUsersResponse {
    data: Vec<TwitchUsersResponseData>
}

#[derive(Deserialize)]
struct TwitchUsersResponseData {
    id: String
}

fn user_id_from_login_name(options: TwitchConnectOptions) -> u32 {
    info!("Fetching channel id for user {:?}", options.channel);

    let mut easy = Easy::new();
    easy.url(format!("https://api.twitch.tv/helix/users?login={}", options.channel).as_ref()).unwrap();

    let mut list = List::new();
    list.append(format!("Authorization: Bearer {}", options.token).as_ref()).unwrap();
    list.append(format!("Client-Id: {}", options.client_id).as_ref()).unwrap();
    easy.http_headers(list).unwrap();

    let mut id_buf: u32 = 0;
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            let TwitchUsersResponse{ data: users } = serde_json::from_slice::<TwitchUsersResponse>(data).unwrap();
            let TwitchUsersResponseData { id } = users.first().unwrap();
            id_buf = id.parse::<u32>().unwrap();
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    info!("Channel id {}", id_buf);
    id_buf
}

async fn create_messages_stream(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    let TwitchConnectOptions { user, token, channel, .. } = options;
    let dispatcher = Dispatcher::new();

    let (mut runner, mut control) = Runner::new(dispatcher.clone());

    let connector = Connector::new(move || {
        let user = user.clone();
        let token = s!("oauth:").to_owned() + &token.clone();
        async move {
            twitchchat::native_tls::connect_easy(&user, &token).await
        }
    });

    tokio::task::spawn(async move {
        runner.run_to_completion(connector).await
    });

    debug!("waiting for irc ready");
    let ready = dispatcher
        .wait_for::<twitchchat::events::IrcReady>()
        .await
        .unwrap();
    info!("Our nickname: {}", ready.nickname);

    let mut writer = control.writer().clone();

    writer.join(channel.clone()).await.unwrap();

    info!("Joined channel: {}", channel);

    dispatcher.clear_subscriptions_all();

    map_events(dispatcher)
}

fn map_events(dispatcher: Dispatcher) -> impl Stream<Item = ChatEvent> {
    let priv_msg = dispatcher.subscribe::<twitchchat::events::Privmsg>();

    priv_msg.map(|msg: Arc<Privmsg>| {
        ChatEvent::Message(ChatMessage {
            name: s!(msg.name),
            content: s!(msg.data),
            is_mod: msg.tags.get("mod").map_or(false, |mod_tag| mod_tag == "1")
                || msg.channel == format!("#{}", msg.name) // weird hack: the owner apparently is not a mod and there is no tag to identify the ownership
        })
    })
}

#[derive(Clone)]
pub struct TwitchConnectOptions {
    pub user: String,
    pub token: String,
    pub channel: String,
    pub client_id: String
}

impl From<TwitchStreamConfig> for TwitchConnectOptions {
    fn from(config: TwitchStreamConfig) -> Self {
        TwitchConnectOptions { user: config.user, token: config.token, channel: config.channel, client_id: config.client_id }
    }
}

impl Display for TwitchConnectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "User: {}, Channel: {}", &self.user, &self.channel)
    }
}
