use dotenv::dotenv;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    model::{
        channel::Message,
        gateway::Ready,
        event::TypingStartEvent,
        misc::Mentionable
    },
    framework::standard::{
        StandardFramework,
        CommandResult,
        macros::{
            command,
            group
        }
    },
    // http::AttachmentType,
};

use std::env;
// use std::path::Path;

#[group]
#[commands(ping, hello)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

#[command]
async fn hello(ctx: &Context, msg: &Message) -> CommandResult {
    let _msg = msg.channel_id.send_message(&ctx.http, |m| {
        m.content("Hello");
        m.embed(|e| {
            e.title("This is a title");
            e.description("This is a description");
            //e.image("attachment://ferris_eyes.png");
            e.fields(vec![
                ("This is the first field", "This is a field body", true),
                ("This is the second field", "Both of these fields are inline", true),
            ]);
            e.field("This is the third field", "This is not an inline field", false);
            e.footer(|f| {
                f.text("This is a footer");

                f
            });
            e
        });

        //m.add_file(AttachmentType::Path(Path::new("./ferris_eyes.png")));
        m
    }).await?;
    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    async fn typing_start(&self, ctx: Context, ev: TypingStartEvent) {
        match ev.user_id.to_user(&ctx.http).await {
            Ok(u) => {
                let _ = ev.channel_id.send_message(&ctx.http, |m| {
                    m.content(format_args!(
                        "{user} est entrain d'écrire... ça sera sûrement très intéressant 🙄",
                        user = u.mention())
                    );
                    m
                }).await;

                tokio::select! {
                    Some(r) = u.await_reaction(&ctx.shard) => {
                        eprintln!("{:?}", r);
                        let _ = ev.channel_id.send_message(&ctx.http, |m|{
                            m.content("Hahahaha !");
                            m
                        }).await;
                    },
                    Some(rep) = u.await_reply(&ctx.shard) => {
                        eprintln!("{:?}", rep);
                        let _ = ev.channel_id.send_message(&ctx.http, |m|{
                            m.content("C'est cela oui !");
                            m
                        }).await;
                    }
                }
            }
            Err(_) => {}
        }
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
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
