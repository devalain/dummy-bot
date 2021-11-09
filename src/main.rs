mod survey;

use crate::survey::{Survey, SurveyKey, SurveyManager, SurveyMap};
use dotenv::dotenv;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args, CommandResult,
        },
        StandardFramework,
    },
    model::{
        channel::{Message, Reaction},
        gateway::Ready,
        id::{ChannelId, MessageId},
    },
    prelude::*,
    Client,
};
use std::{env, sync::Arc};

#[group]
#[commands(ping, survey)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(name) = msg.channel_id.name(&ctx.cache).await.as_deref() {
        msg.reply(ctx, format!("Pong in '{}' ({})", name, msg.channel_id))
            .await?;
    } else {
        msg.reply(ctx, "Pong !").await?;
    }
    Ok(())
}

#[command]
async fn survey(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let question = args.single_quoted::<String>()?; // Gets the first "argument"
    let answers = args
        .quoted() //Enables "quoting" for answers with space
        .iter::<String>() //Iterates over the rest of the arguments
        .filter_map(|x| x.ok()) //Filters out any argument that failed to parse
        .collect::<Vec<_>>(); //Collects all the arguments into a Vec<String>
    let survey = Survey::new(question, &answers);
    ctx.new_survey(msg.channel_id, survey).await?;
    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        println!("Added reaction");
        if add_reaction.user_id != Some(ctx.cache.current_user_id().await) {
            match ctx.update_survey_add(&add_reaction).await {
                Ok(()) => println!("Sucess !"),
                Err(e) => println!("Error: {:#?}", e),
            }
        }
    }

    async fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        println!("Removed reaction");
        if removed_reaction.user_id != Some(ctx.cache.current_user_id().await) {
            match ctx.update_survey_rm(&removed_reaction).await {
                Ok(()) => println!("Sucess !"),
                Err(e) => println!("Error: {:#?}", e),
            }
        }
    }

    async fn reaction_remove_all(
        &self,
        _ctx: Context,
        _channel_id: ChannelId,
        _removed_from_message_id: MessageId,
    ) {
        println!("All reactions removed");
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenv();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<SurveyKey>(Arc::new(Mutex::new(SurveyMap::new())))
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
