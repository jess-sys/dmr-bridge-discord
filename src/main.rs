use serenity::{
    client::Client,
    framework::{
        StandardFramework,
    }
};

use songbird::{
    driver::DecodeMode,
    Config,
    SerenityInit,
};

use dotenv::dotenv;
use std::env;

mod handler;
mod commands;

use handler::Handler;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("BOT_TOKEN")
        .expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c
            .prefix(env::var("BOT_PREFIX")
                .expect("Expected a prefix in the environment").as_str()))
        .group(&commands::GENERAL_GROUP);

    let songbird_config = Config::default()
        .decode_mode(DecodeMode::Decode);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird_from_config(songbird_config)
        .await
        .expect("Error creating client");

    let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
}