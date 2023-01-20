use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::channel::Message,
    prelude::{Mutex, TypeMapKey, Mentionable},
    Result as SerenityResult,
};

use songbird::CoreEvent;

use std::sync::Arc;

mod transmitter;
pub mod receiver;

use transmitter::{Transmitter, TransmitterWrapper};
use chrono::prelude::Utc;
use receiver::Receiver;

pub struct DMRContext;

impl TypeMapKey for DMRContext {
    type Value = Arc<Mutex<Receiver>>;
}

#[group]
#[commands(join, leave, ping)]
pub struct General;

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let channel = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "⚠️ Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (handler_lock, conn_result) = manager.join(guild_id, channel).await;

    if conn_result.is_ok() {
        // NOTE: this skips listening for the actual connection result.
        let mut handler = handler_lock.lock().await;

        let transmitter = Arc::new(Transmitter::new());
        let speaking_update_transmitter = TransmitterWrapper::new(transmitter.clone());
        let voice_packet_transmitter = TransmitterWrapper::new(transmitter.clone());

        handler.add_global_event(CoreEvent::SpeakingUpdate.into(), speaking_update_transmitter);
        handler.add_global_event(CoreEvent::VoicePacket.into(), voice_packet_transmitter);

        let receiver_lock = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<DMRContext>()
                .expect("Expected DMRContext in TypeMap.")
                .clone()
        };

        {
            let mut receiver = receiver_lock.lock().await;
            receiver.set(handler_lock.clone());
        }

        check_msg(
            msg.reply(ctx, &format!("Joined {}", channel.mention()))
                .await,
        );
    } else {
        check_msg(msg.reply(ctx, "Error joining the channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
        .unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        let receiver_lock = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<DMRContext>()
                .expect("Expected DMRContext in TypeMap.")
                .clone()
        };

        {
            let mut receiver = receiver_lock.lock().await;
            receiver.unset();
        }

        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.reply(ctx, format!("Failed: {:?}", e)).await);
        }
        check_msg(
            msg.reply(ctx, &format!("Left {}", channel_id.mention()))
                .await,
        );
    } else {
        check_msg(msg.reply(ctx, "⚠️ Not in a voice channel").await);
    }

    Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let now = Utc::now();
    let elapsed = now - *msg.timestamp;
    check_msg(
        msg.reply(ctx, format!("Pong! ({} ms)", elapsed.num_milliseconds()))
            .await,
    );

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
