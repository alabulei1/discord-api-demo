// use discord_flows::model::prelude::*;
use discord_flows::{
    http::HttpBuilder, model::application_command::CommandDataOptionValue, Bot, EventModel,
    ProvidedBot,
};
use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use serde::Deserialize;
use serde_json;
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    dotenv().ok();
    logger::init();
    let discord_token = env::var("discord_token").unwrap();
    let commands_registered = env::var("COMMANDS_REGISTERED").unwrap_or("false".to_string());

    match commands_registered.as_str() {
        "false" => {
            register_commands(&discord_token).await;
            env::set_var("COMMANDS_REGISTERED", "true");
        }
        _ => {}
    }
    let bot = ProvidedBot::new(discord_token);
    bot.listen(|em| handle(&bot, em)).await;
}

async fn register_commands(discord_token: &str) {
    let bot_id = env::var("bot_id").unwrap_or("1124137839601406013".to_string());
    // let channel_id = env::var("discord_channel_id").unwrap_or("1128056246570860617".to_string());
    //     let guild_id = env::var("discord_guild_id").unwrap_or("1128056245765558364".to_string());
    //     // Define the Discord API endpoint for registering commands
    //     let uri = format!(
    //         "https://discord.com/api/v8/applications/{}/guilds/{}/commands",
    //         bot_id, guild_id
    //     );

    let command = serde_json::json!({
        "name": "weather",
        "description": "Get the weather for a city",
        "options": [
            {
                "name": "city",
                "description": "The city to lookup",
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
        Ok(_) => log::info!("Successfully registered command"),
        Err(err) => log::error!("Error registering command: {}", err),
    }
}
async fn handle<B: Bot>(bot: &B, em: EventModel) {
    match em {
        EventModel::ApplicationCommand(ac) => {
            let client = bot.get_client();
            let channel_id = ac.channel_id.as_u64();
            match ac.data.name.as_str() {
                "weather" => {
                    let options = ac
                        .data
                        .options
                        .get(0)
                        .expect("Expected city option")
                        .resolved
                        .as_ref()
                        .expect("Expected city object");

                    let city = match options {
                        CommandDataOptionValue::String(s) => s,
                        _ => panic!("Expected string for city"),
                    };

                    let resp_inner = match get_weather(&city) {
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
                    let resp = serde_json::json!(
                        {
                            "content": resp_inner
                        }
                    );
                    _ = client.send_message(*channel_id, &resp).await;
                }
                _ => {}
            }
        }
        EventModel::Message(msg) => {
            let client = bot.get_client();
            let channel_id = msg.channel_id;
            let content = msg.content;
        }
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
