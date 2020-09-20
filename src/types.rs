use std::sync::Arc;

use serenity::{client::bridge::gateway::ShardManager, model::gateway::Ready, prelude::*};

use time::Instant;

/// OxiBot event handler
pub struct OxiHandler;

impl EventHandler for OxiHandler {
    #[inline]
    fn ready(&self, _ctx: Context, ready: Ready) {
        // Reset every time it reconnects
        // SAFETY: safe because it's the only other place where we mutate the `static mut`
        unsafe { *crate::UPTIME = Instant::now() };

        println!("{} is connected!", ready.user.name)
    }
}

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}
