use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        CommandResult
    },
    model::{
        channel::Message,
        misc::Mentionable
    },
    Result as SerenityResult,
    prelude::{TypeMapKey, Mutex}
};

use songbird::CoreEvent;

use std::sync::Arc;

mod receiver;
pub mod transmitter;

use chrono::prelude::Utc;
use receiver::Receiver;
use transmitter::Transmitter;

pub struct DMRContext;

impl TypeMapKey for DMRContext {
    type Value = Arc<Mutex<Transmitter>>;
}

#[group]
#[commands(join, leave, ping)]
pub struct General;

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let channel = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "⚠️ Not in a voice channel").await);

            return Ok(())
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let (handler_lock, conn_result) = manager.join(guild_id, channel).await;

    if let Ok(_) = conn_result {
        // NOTE: this skips listening for the actual connection result.
        let mut handler = handler_lock.lock().await;
        let receiver = Arc::new(Receiver::new());

        handler.add_global_event(
            CoreEvent::VoicePacket.into(),
            *receiver.clone(),
        );

        handler.add_global_event(
            CoreEvent::SpeakingUpdate.into(),
            *receiver.clone(),
        );


        let transmitter_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<DMRContext>().expect("Expected DMRContext in TypeMap.").clone()
        };

        {
            let mut transmitter = transmitter_lock.lock().await;
            transmitter.set( handler_lock.clone());
        }

        check_msg(msg.reply(ctx, &format!("Joined {}", channel.mention())).await);
    } else {
        check_msg(msg.reply(ctx, "Error joining the channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
        .unwrap();

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        let transmitter_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<DMRContext>().expect("Expected DMRContext in TypeMap.").clone()
        };

        {
            let mut transmitter = transmitter_lock.lock().await;
            transmitter.unset();
        }

        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.reply(ctx, format!("Failed: {:?}", e)).await);
        }
        check_msg(msg.reply(ctx, &format!("Left {}", channel_id.mention())).await);
    } else {
        check_msg(msg.reply(ctx, "⚠️ Not in a voice channel").await);
    }

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let now = Utc::now();
    let elasped = now - msg.timestamp;
    check_msg(msg.reply(ctx, format!("Pong! ({} ms)", elasped.num_milliseconds())).await);

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}