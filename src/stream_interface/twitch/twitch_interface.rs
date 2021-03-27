use std::fmt::{Display, Error, Formatter};
use futures::stream::Stream;
use tokio::sync::mpsc::channel;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use crate::s;
use crate::stream_interface::events::{ChatEvent, ChatMessage};
use crate::utils::app_config::TwitchStreamConfig;
use crate::stream_interface::twitch::channel_events_stream::create_channel_events_stream;

pub async fn connect_to_twitch(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    info!("Connecting to stream: {}", options);
    let chat_stream = create_messages_stream(options.clone()).await;
    let channel_rewards_stream = create_channel_events_stream(options).await;
    chat_stream.merge(channel_rewards_stream)
}

async fn create_messages_stream(options: TwitchConnectOptions) -> impl Stream<Item = ChatEvent> {
    let TwitchConnectOptions { user, token, channel: channel_to_log_into, .. } = options;
    let config = ClientConfig::new_simple(StaticLoginCredentials::new(user, Some(token)));
    let (mut incoming_messages, client) =
        TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(config);
    
    let (tx, rx) = channel::<ChatEvent>(100);

    tokio::spawn(async move {
        let join_handle = tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                if let ServerMessage::Privmsg(msg) = message {
                    debug!("Irc Private Message received {:?}", msg);
                    let has_mod_tag;
                    match msg.source.tags.0.get("mod") {
                        Some(Some(value)) => has_mod_tag = value == "1",
                        _ => has_mod_tag = false
                    };

                    let has_broadcaster_badge = msg.badges.into_iter().any(|badge| badge.name == "broadcaster");
                    tx.send(ChatEvent::Message(ChatMessage {
                        name: s!(msg.sender.name),
                        content: s!(msg.message_text),
                        is_mod: has_mod_tag || has_broadcaster_badge
                    })).await.unwrap();
                } else {
                    debug!("Irc message that is not a Private Message {:?}", message);
                }
            }
        });

        client.join(channel_to_log_into);

        join_handle.await.unwrap();
    });
    
    ReceiverStream::new(rx)
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
