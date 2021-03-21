use curl::easy::{Easy, List};
use serde::{Deserialize};
use crate::stream_interface::twitch::twitch_interface::TwitchConnectOptions;

#[derive(Deserialize)]
struct TwitchUsersResponse {
    data: Vec<TwitchUsersResponseData>
}

#[derive(Deserialize)]
struct TwitchUsersResponseData {
    id: String
}

pub fn user_id_from_login_name(options: TwitchConnectOptions) -> u32 {
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