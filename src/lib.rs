// use discord_flows::model::prelude::*;
use discord_flows::{
    model::{interaction, Interaction, InteractionType, Message},
    Bot, ProvidedBot,
};
use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use serde::Deserialize;
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    dotenv().ok();
    let discord_token = env::var("discord_token").unwrap();
    let bot = ProvidedBot::new(discord_token);
    bot.listen(|msg| handler(&bot, msg)).await;
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
        Some(interaction) => match &interaction.kind {
            InteractionType::ApplicationCommand => {
                match interaction.name.as_str() {
                    "weather" => {
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
        },
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
// if let Some(interaction) = msg.interaction {
//     if &interaction.kind == &InteractionType::Ping {
//         let pong = serde_json::json!({
//             "type": 1
//         });

//         // let pong_str = serde_json::to_string(&pong).unwrap_or_default();

//         // send_response(
//         //     200,
//         //     vec![(
//         //         String::from("content-type"),
//         //         String::from("application/json"),
//         //     )],
//         //     pong_str.as_bytes().to_vec(),
//         // );

//         let res = discord
//             .send_message(
//                 channel_id.into(),
//                 &serde_json::json!({
//                     "content": pong,
//                 }),
//             )
//             .await;
//     }

//     let name = interaction.name;
//     // let pong = serde_json::json!({
//     //     "type": 1
//     // });

//     // let pong_str = serde_json::to_string(&pong).unwrap_or_default();

//     // send_response(
//     //     200,
//     //     vec![(
//     //         String::from("content-type"),
//     //         String::from("application/json"),
//     //     )],
//     //     pong_str.as_bytes().to_vec(),
//     // );

//     let res = discord
//         .send_message(
//             channel_id.into(),
//             &serde_json::json!({
//                 "content": name,
//             }),
//         )
//         .await;
// }

// let bot_name = std::env::var("BOT_NAME").unwrap_or(String::from("Chat assistant"));

// _ = discord
//     .edit_profile(
//         serde_json::json!({ "username": bot_name })
//             .as_object()
//             .unwrap(),
//     )
//     .await;

// let co = ChatOptions {
//     model: ChatModel::GPT35Turbo,
//     restart: false,
//     system_prompt: Some("You are a helpful assistant answering questions on Discord. If someone greets you without asking a question, you should simply respond \"Hello, I am your assistant on Discord, built by the Second State team. I am ready for your instructions now!\""),
//     max_tokens: Some(150),
//     ..Default::default()
// };

// let of = OpenAIFlows::new();

// if let Ok(c) = of
//     .chat_completion(&channel_id.to_string(), &content, &co)
//     .await
// {
//     _ = discord
//         .send_message(
//             channel_id.into(),
//             &serde_json::json!({
//                 "content": c.choice,
//             }),
//         )
//         .await;
// }
