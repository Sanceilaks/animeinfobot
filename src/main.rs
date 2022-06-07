mod animeinfo;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot && msg.content.starts_with("v_") && msg.content.len() > 2{
            let content = msg.content[2..].to_string();

            let mut args = content.split_whitespace();
            let command: std::string::String = args.next().unwrap().to_string();

            if command == "animeinfo" {
                let mut waitmsg = msg.reply(&ctx, "Поиск аниме...").await.unwrap(); 

                animeinfo::anime_info(&ctx, &mut waitmsg, args.collect()).await.unwrap();
            }
            if command == "watch" {
                msg.reply(&ctx, 
                    "Видео можно смотреть с его страницы на шикимори\nПодробнее: https://github.com/Smarthard/shikicinema"
                ).await.unwrap();
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("Bot started: {}", ready.user.name);
    }
}

#[tokio::main]
async fn main(){
    tracing_subscriber::fmt().try_init().expect("Cannot init tracing");

    let config: serde_json::Value = serde_json::from_str(
        String::from_utf8_lossy(&std::fs::read("botconfig.json").expect("Cannot find config file")).into_owned().as_str(),
    ).unwrap();

    let mut client = 
        Client::builder(config["token"].as_str().unwrap(), 
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::DIRECT_MESSAGES
    ).event_handler(Handler).await.expect("Error creating client");
    
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
