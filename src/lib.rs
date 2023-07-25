// use discord_flows::model::prelude::*;
use discord_flows::{model::Message, Bot, ProvidedBot};
use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use openai_flows::{
    chat::{ChatModel, ChatOptions},
    OpenAIFlows,
};
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    let discord_token = env::var("discord_token").unwrap();
    let bot = ProvidedBot::new(discord_token);
    bot.listen(|msg| handler(&bot, msg)).await;
    Ok(())
}

async fn handler(bot: &ProvidedBot, msg: Message) {
    logger::init();
    let discord = bot.get_client();
    let channel_id = msg.channel_id;
    // let discord_guild_id =
    //     env::var("discord_guild_id").unwrap_or("1128056245765558364".to_string());


    if msg.author.bot {
        log::debug!("ignored bot message");
        return;
    }
    if msg.member.is_some() {
        log::debug!("ignored channel message");
        return;
    }
    let resp = format!("Welcome to flows.network.\nYou just said: '{}'.\nLearn more at: https://github.com/flows-network/hello-world\n", msg.content);

    _ = discord
        .send_message(channel_id.into(), &serde_json::json!({ "content": resp }))
        .await;

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
}
