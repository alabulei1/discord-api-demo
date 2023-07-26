// use discord_flows::model::prelude::*;
use discord_flows::{
    model::{interaction, Interaction, InteractionType, Message},
    Bot, ProvidedBot,
};
use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use http_req::{
    request::{Method, Request},
    response::{Headers, Response},
    uri::Uri,
};
use unicase::Ascii;

use serde::Deserialize;
use serde_json::Value;
use std::env;
use std::io::Write;
#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    dotenv().ok();
    let discord_token = env::var("discord_token").unwrap();
    let bot = ProvidedBot::new(discord_token.clone());

    // Register slash commands
    register_commands(&bot, &discord_token).await?;

    bot.listen(|msg| handler(&bot, msg)).await;
    Ok(())
}

// Register slash commands
async fn register_commands(bot: &ProvidedBot, discord_token: &str) -> anyhow::Result<()> {
    // Define the bot_id and guild_id for the command to be registered to
    let bot_id = env::var("bot_id").unwrap_or("1124137839601406013".to_string());
    let guild_id = env::var("discord_guild_id").unwrap_or("1128056245765558364".to_string());
    let channel_id = env::var("discord_channel_id").unwrap_or("1128056246570860617".to_string());
    let discord = bot.get_client();
    // Define the Discord API endpoint for registering commands
    let uri = format!(
        "https://discord.com/api/v8/applications/{}/guilds/{}/commands",
        bot_id, guild_id
    );

    // Define the slash command
    // let command = serde_json::json!({
    //     "name": "weather",
    //     "description": "Get the weather for a city",
    //     "options": [
    //         {
    //             "name": "city",
    //             "description": "The city to get the weather for",
    //             "type": 3,
    //             "required": true
    //         }
    //     ]
    // })
    // .to_string();
    let command = serde_json::json!({
        "name": "fake",
        "description": "This is a fake command",
        "options": [
            {
                "name": "nothing",
                "description": "Nothing to go for",
                "type": 3,
                "required": true
            }
        ]
    })
    .to_string();

    let mut headers = Headers::new();
    headers.insert(
        &Ascii::new("Authorization"),
        &format!("Bot {}", discord_token),
    );
    headers.insert(&Ascii::new("Content-Type"), &"application/json".to_string());
    headers.insert(
        &Ascii::new("Content-Length"),
        &command.as_bytes().len().to_string(),
    );
    let mut writer = Vec::new();
    post(&uri, headers, command.as_bytes(), &mut writer)?;

    let response = Response::from_head(&writer)?;
    if response.status_code().is_success() {
        _ = discord
            .send_message(
                channel_id.parse::<u64>().unwrap(),
                &serde_json::json!({ "content": "Successfully registered command" }),
            )
            .await;
    } else {
        _ = discord
            .send_message(
                channel_id.parse::<u64>().unwrap(),
                &serde_json::json!({
                    "content": &format!("Failed to register command: {}", response.status_code())
                }),
            )
            .await;
        &format!("Failed to register command: {}", response.status_code());
    }

    Ok(())
}

pub fn post<T: AsRef<str>, U: Write>(
    uri: T,
    headers: Headers,
    body: &[u8],
    writer: &mut U,
) -> Result<Response, http_req::error::Error> {
    let uri = Uri::try_from(uri.as_ref())?;

    Request::new(&uri)
        .method(Method::POST)
        .headers(headers)
        .body(body)
        .send(writer)
}

async fn handler(bot: &ProvidedBot, msg: Message) {
    logger::init();
    let discord = bot.get_client();
    let channel_id = msg.channel_id;
    log::info!("channel_id: {:?}", channel_id);
    if msg.author.bot {
        log::debug!("ignored bot message");
        return;
    }

    match msg.interaction {
        Some(interaction) => {
            match &interaction.kind {
                InteractionType::ApplicationCommand => {
                    match interaction.name.as_str() {
                        "weather" => {
                            // let city_option = interaction
                            //     .data
                            //     .options
                            //     .into_iter()
                            //     .find(|option| option.name == "city");

                            // let city = match city_option.and_then(|option| option.value) {
                            //     Some(Value::String(city)) => city,
                            //     _ => {
                            //         log::error!("Invalid or missing 'city' option");
                            //         return;
                            //     }
                            // };

                            let city = msg.content.trim_start_matches("/weather").trim();
                            log::info!("city: {}", city);
                            let resp = match get_weather(&city) {
                                Some(w) => format!(
                                    r#"Today: {},
        Low temperature: {} °C,
        High temperature: {} °C,
        Wind Speed: {} km/h"#,
                                    w.weather
                                        .first()
                                        .unwrap_or(&Weather {
                                            main: "Unknown".to_string()
                                        })
                                        .main,
                                    w.main.temp_min as i32,
                                    w.main.temp_max as i32,
                                    w.wind.speed as i32
                                ),
                                None => String::from("No city or incorrect spelling"),
                            };
                            _ = discord
                                .send_message(
                                    channel_id.into(),
                                    &serde_json::json!({ "content": &resp }),
                                )
                                .await;
                        }
                        _ => {}
                    };
                }
                _ => {}
            }
        }
        None => {}
    }
}

#[derive(Deserialize, Debug)]
struct ApiResult {
    weather: Vec<Weather>,
    main: Main,
    wind: Wind,
}

#[derive(Deserialize, Debug)]
struct Weather {
    main: String,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp_max: f64,
    temp_min: f64,
}

#[derive(Deserialize, Debug)]
struct Wind {
    speed: f64,
}

fn get_weather(city: &str) -> Option<ApiResult> {
    let mut writer = Vec::new();
    let api_key = env::var("API_KEY").unwrap_or("fake_api_key".to_string());
    let query_str = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={city}&units=metric&appid={api_key}"
    );

    let uri = Uri::try_from(query_str.as_str()).unwrap();
    match Request::new(&uri).method(Method::GET).send(&mut writer) {
        Err(_e) => log::error!("Error getting response from weather api: {:?}", _e),

        Ok(res) => {
            if !res.status_code().is_success() {
                log::error!("weather api http error: {:?}", res.status_code());
                return None;
            }
            match serde_json::from_slice::<ApiResult>(&writer) {
                Err(_e) => log::error!("Error deserializing weather api response: {:?}", _e),
                Ok(w) => {
                    log::info!("Weather: {:?}", w);
                    return Some(w);
                }
            }
        }
    };
    None
}
