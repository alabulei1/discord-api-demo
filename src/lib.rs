// use discord_flows::model::prelude::*;
use discord_flows::{
    http::HttpBuilder,
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
use tokio::time::{sleep, Duration};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    dotenv().ok();
    let discord_token = env::var("discord_token").unwrap();
    let bot = ProvidedBot::new(discord_token.clone());
    let commands_registered = env::var("COMMANDS_REGISTERED").unwrap_or("false".to_string());

    if commands_registered == "false" {
        register_commands(&bot, &discord_token).await?;
        env::set_var("COMMANDS_REGISTERED", "true");
    }
    bot.listen(|msg| handler(&bot, msg)).await;

    Ok(())
}

async fn register_commands(bot: &ProvidedBot, discord_token: &str) -> anyhow::Result<()> {
    let bot_id = env::var("bot_id").unwrap_or("1124137839601406013".to_string());
    let channel_id = env::var("discord_channel_id").unwrap_or("1128056246570860617".to_string());
    //     let guild_id = env::var("discord_guild_id").unwrap_or("1128056245765558364".to_string());
    //     // Define the Discord API endpoint for registering commands
    //     let uri = format!(
    //         "https://discord.com/api/v8/applications/{}/guilds/{}/commands",
    //         bot_id, guild_id
    //     );
    let discord = bot.get_client();

    let command = serde_json::json!({
        "name": "weather",
        "description": "Get the weather for a city",
        "options": [
            {
                "name": "city",
                "description": "The city to get the weather for",
                "type": 3,
                "required": true
            }
        ]
    });

    let http_client = HttpBuilder::new(discord_token)
        .application_id(bot_id.parse().unwrap())
        .build();

    match http_client
        .create_global_application_command(&command)
        .await
    {
        Ok(_) => {
            _ = discord
                .send_message(
                    channel_id.parse::<u64>().unwrap(),
                    &serde_json::json!({ "content": "Successfully registered command" }),
                )
                .await;
        }
        Err(err) => {
            _ = discord
                .send_message(
                    channel_id.parse::<u64>().unwrap(),
                    &serde_json::json!({
                        "content": &format!("Failed to register command: {}", err)
                    }),
                )
                .await;
        }
    }

    Ok(())
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
                            // let city_option = command
                            //     .data
                            //     .options
                            //     .get(0)
                            //     .expect("Expected city option")
                            //     .resolved
                            //     .as_ref()
                            //     .expect("Expected city object");
                            _=    discord
                            .send_message(
                                channel_id.into(),
                                &serde_json::json!({ "content": "Please provide a valid city" }),
                            )
                            .await;
                            //                 match city_option {
                            //                 CommandDataOptionValue::String(city) => {
                            //                     log::info!("city: {}", city);
                            //                     let resp = match get_weather(&city) {
                            //                         Some(w) => format!(
                            //                             r#"Today: {},
                            // Low temperature: {} °C,
                            // High temperature: {} °C,
                            // Wind Speed: {} km/h"#,
                            //                             w.weather
                            //                                 .first()
                            //                                 .unwrap_or(&Weather {
                            //                                     main: "Unknown".to_string()
                            //                                 })
                            //                                 .main,
                            //                             w.main.temp_min as i32,
                            //                             w.main.temp_max as i32,
                            //                             w.wind.speed as i32
                            //                         ),
                            //                         None => String::from("No city or incorrect spelling"),
                            //                     };
                            //                     _ = discord
                            //                         .send_message(
                            //                             channel_id.into(),
                            //                             &serde_json::json!({ "content": &resp }),
                            //                         )
                            //                         .await;
                            //                 },
                            //                 _ =>     _=    discord
                            //                 .send_message(
                            //                     channel_id.into(),
                            //                     &serde_json::json!({ "content": "Please provide a valid city" }),
                            //                 )
                            //                 .await,
                            //             }

                            // let city = msg.content.trim_start_matches("/weather").trim();
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
