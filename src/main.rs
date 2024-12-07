use std::thread::sleep;
use std::time;

use discord_rich_presence::{activity, activity::ActivityType, DiscordIpc, DiscordIpcClient};
use reqwest::Client;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    let client = Client::new();
    let username: &str = &std::env::var("LASTFM_USERNAME").unwrap();
    let apikey: &str = &std::env::var("LASTFM_APIKEY").unwrap();
    let discord_client_id: &str = &std::env::var("DISCORD_CLIENTID").unwrap();
    let mut old_track = serde_json::Value::Null;

    let mut discord_client = DiscordIpcClient::new(discord_client_id).unwrap();
    discord_client.connect().unwrap();
    loop {
        let response = get_api(&client, username, apikey).unwrap();
        let recent_tracks = &response["recenttracks"]["track"];
        let now_playing = &recent_tracks[0];
        if now_playing.get("@attr").is_some()
            && now_playing["@attr"].get("nowplaying").is_some()
            && now_playing != &old_track
        {
            let track = now_playing["name"].as_str().unwrap();
            let artist = now_playing["artist"]["#text"].as_str().unwrap();

            println!("Now playing: {} by {}", track, artist);
            discord_client
                .set_activity(
                    activity::Activity::new()
                        .details(track)
                        .activity_type(ActivityType::Listening)
                        .assets(
                            activity::Assets::new()
                                .large_image(now_playing["image"][3]["#text"].as_str().unwrap())
                                .large_text(artist),
                        )
                        .timestamps(
                            activity::Timestamps::new().start(
                                time::SystemTime::now()
                                    .duration_since(time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64,
                            ),
                        ),
                )
                .unwrap();

            old_track = now_playing.clone();
        }
        sleep(time::Duration::from_secs(5));
    }
    //discord_client.close().unwrap();
}

#[tokio::main]
async fn get_api(client: &Client, username: &str, apikey: &str) -> Result<serde_json::Value> {
    let url = format!("http://ws.audioscrobbler.com/2.0");
    let response = client
        .get(url)
        .query(&[
            ("method", "user.getrecenttracks"),
            ("user", username),
            ("api_key", apikey),
            ("format", "json"),
        ])
        .send()
        .await?;
    Ok(response.json().await?)
}
