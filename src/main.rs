use discord_rich_presence::{activity, activity::ActivityType, DiscordIpc, DiscordIpcClient};
use reqwest::Client;
use std::thread::sleep;
use std::time;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn main() {
    dotenv::dotenv().ok();

    let username: &str = &std::env::var("LASTFM_USERNAME").unwrap();
    let api_key: &str = &std::env::var("LASTFM_APIKEY").unwrap();
    let client_id: &str = &std::env::var("DISCORD_CLIENTID").unwrap();

    // Discordに接続
    let mut discord_client = DiscordIpcClient::new(client_id).unwrap();
    discord_client.connect().unwrap();

    let client = Client::new();
    let mut old_track = serde_json::Value::Null;
    loop {
        let response = get_api(&client, username, api_key).unwrap();
        let recent_tracks = &response["recenttracks"]["track"];

        if recent_tracks[0]["@attr"].get("nowplaying").is_some() && &old_track != recent_tracks {
            let artist = recent_tracks[0]["artist"]["#text"].as_str().unwrap();
            let track = recent_tracks[0]["name"].as_str().unwrap();

            let artist_picture = get_artist_picture(&client, artist).unwrap();

            println!("Now playing: {} - {}", artist, track);
            update_presence(&mut discord_client, recent_tracks, &artist_picture);
            old_track = recent_tracks.clone();
        }
        sleep(time::Duration::from_secs(2));
    }
}

fn update_presence(
    discord_client: &mut DiscordIpcClient,
    data: &serde_json::Value,
    artist_picture: &serde_json::Value,
) {
    if artist_picture == &serde_json::Value::Null {
        discord_client
            .set_activity(
                activity::Activity::new()
                    .details(data[0]["name"].as_str().unwrap())
                    .activity_type(ActivityType::Listening)
                    .assets(
                        activity::Assets::new()
                            .large_image(data[0]["image"][3]["#text"].as_str().unwrap())
                            .large_text(data[0]["artist"]["#text"].as_str().unwrap()),
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
    } else {
        discord_client
            .set_activity(
                activity::Activity::new()
                    .details(data[0]["name"].as_str().unwrap())
                    .activity_type(ActivityType::Listening)
                    .assets(
                        activity::Assets::new()
                            .large_image(data[0]["image"][3]["#text"].as_str().unwrap())
                            .large_text(data[0]["artist"]["#text"].as_str().unwrap())
                            .small_image(artist_picture["picture_small"].as_str().unwrap()),
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
    }
}

#[tokio::main]
async fn get_artist_picture(client: &Client, artist: &str) -> Result<serde_json::Value> {
    let url = format!("https://api.deezer.com/search/artist");
    let response: serde_json::Value = client
        .get(url)
        .query(&[("q", artist)])
        .send()
        .await?
        .json()
        .await?;

    if &response["total"] == 0 {
        return Ok(serde_json::Value::Null);
    } else {
        let _artist = artist.to_lowercase();
        for i in 0..response["data"].as_array().unwrap().len() {
            let name = response["data"][i]["name"].as_str().unwrap().to_lowercase();
            if name == _artist {
                return Ok(response["data"][i].clone());
            }
        }
    }

    Ok(serde_json::Value::Null)
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
            ("limit", "1"),
        ])
        .send()
        .await?;
    Ok(response.json().await?)
}
