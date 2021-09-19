use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::gateway::Ready,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}