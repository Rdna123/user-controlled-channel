use std::env;
use std::thread::sleep;
use std::time::Duration;
use dotenv::dotenv;

use itertools::Itertools;
use serenity::async_trait;
use serenity::model::prelude::{ChannelType, Role, ReactionType, GuildChannel, ChannelId};
// use serenity::model::prelude::Guild;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};

#[group]
#[commands(ping, quit, channel_delete, controlled_channel)]
struct General;

// #[group]
// #[commands(channel_delete)]
// struct Admin;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn channel_delete(&self, ctx: Context, channel: &GuildChannel) {
        let name = channel.name.clone();
        let guild = channel.guild(&ctx).unwrap();
        let role_id_op = guild.role_by_name(&name);
        let role_id = match role_id_op {
            Some(x) => {x},
            None =>{
                println!("No role for deleted channel");
                return;
            }
        };
        guild.delete_role(&ctx, role_id).await.unwrap();
        println!("role delete");
    }
}
 
#[tokio::main]
async fn main() {
    dotenv().ok();
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::all();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    msg.delete(ctx).await?;
   
    Ok(())
}

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult{
    if msg.content.contains("yes") {
        msg.reply(ctx, "Quiting!").await?;
        std::process::exit(0);
    }
    msg.reply(ctx, "If you wish to kill this bot do 'quit yes'").await?;
    
    Ok(())
}

#[command]
#[only_in(guild)]
#[required_permissions("ADMINISTRATOR")]
async fn controlled_channel(ctx: &Context, msg: &Message)->CommandResult{
    println!("Creating controlled channel");

    //is admin

    let users = msg.mentions.clone();

    let guild = msg.guild(ctx).unwrap();
    let guildid = msg.guild_id.unwrap();

    let cat = ctx.cache.guild_categories(guildid).unwrap();
    let itcat = cat.iter();
    
    // let itcat_name_id = itcat.map(|c| (c.name.clone(), c.id.clone()));
    

    let cat_name = itcat.clone().map(|c| c.name.clone()).position(|p| p == "controlled");

    let itcat_id = itcat.map(|c| c.id.clone());
    let vcat_id = itcat_id.collect_vec();

    

    let con_cat_id:Option<ChannelId>;

    //Make own category
    if cat_name.is_none(){
        let id = ctx.cache.guild(guild.id).unwrap().create_channel(ctx, |f| f.name("controlled").kind(ChannelType::Category)).await?;
        con_cat_id = Some(id.id);
    } else {
        con_cat_id = Some(vcat_id[cat_name.unwrap()]);
    }


    // Channel Check
    let chn = ctx.cache.guild_channels(guildid).unwrap();
    let itchn = chn.iter().map(|x|x.name.clone());
    let vchn = itchn.collect_vec();
    
    // Role Check
    let rle = ctx.cache.guild_roles(guildid).unwrap();
    let itrle = rle.iter().map(|r| r.1.name.clone());
    let vrle = itrle.collect_vec();

    println!("Setup done");

    let mut delmsg: Vec<Option<Message>> = vec!(None);

    for u in users{
        let name = format!("controller-{}",u.name.to_lowercase());
        if !vchn.contains(&name){
            guild.create_channel(ctx, |c| c.name(name.clone()).kind(ChannelType::Text).category(con_cat_id.unwrap())).await?;
        }

        if !vrle.contains(&name){
            let role = guild.create_role(ctx, |c| c.name(name)).await?;
        
            let mut  member = guild.id.member(ctx, u.id).await?;
            member.add_role(ctx, role.id).await?;
            println!("Role add");
        }
        let m = msg.reply(ctx, format!("{} your channel is ready", u.mention())).await?;
        delmsg.push(Some(m));
    }

    println!("Message send");

    sleep(Duration::from_secs(5));

    for m in delmsg {
        match m {
            Some(x) => x.delete(ctx).await?,
            _ => {}
        }
    }
    msg.delete(ctx).await?;
    
    println!("Channel created");
    
    Ok(())
}

#[command]
#[required_permissions("ADMINISTRATOR")]
#[only_in(guild)]
async fn channel_delete(ctx: &Context, msg: &Message)->CommandResult{
    let itroles = ctx.cache.guild_roles(msg.guild_id.unwrap()).unwrap();
    let roles = itroles.iter().map(|x| x.1);

    let mut role: Option<&Role> = None;
    for r in roles{
        if format!("controller-{}", msg.author.name.to_lowercase()) == r.name {
            role = Some(r);
            break;
        };
    };
    
    if role.is_none(){
        msg.reply(ctx, "Role could not be found for this channel").await?;
        return Ok(());
    }
    
    if  !(msg.channel_id.name(ctx).await.unwrap()== role.unwrap().name){
        msg.reply(ctx, format!("You do not have permisson to use this command in {}", msg.channel_id.name(ctx).await.unwrap())).await?;
        return Ok(());
    };

    // println!("Reaction");
    msg.react(ctx, ReactionType::Unicode("👍".to_string())).await?;
    
    // check if channal and user mactch
    let id = msg.content.strip_prefix("~channel_delete").unwrap().split_whitespace();
    
    //let log = itertools::id.clone().intersperse(&", ");
    let log = itertools::intersperse(id.clone(), ", ");
    println!("messages {} deleted from {}",log.collect::<String>(), msg.guild_id.unwrap().name(ctx).unwrap());


    let num_id = id.map(|x| x.parse::<u64>().unwrap());
    for n in num_id{
        msg.channel_id.delete_message(ctx, n).await?;    
    }

    msg.delete(ctx).await?;

    Ok(())
}

// #[command]
// async fn guild_make(ctx: &Context, msg: &Message)->CommandResult{
//     let guild = Guild::create(ctx, "dave", None).await?;
//     let inv = guild.channel_id_from_name(ctx, "general").unwrap().create_invite(ctx, |i| i.max_age(3600).max_uses(10)).await?;
//     msg.reply(ctx, format!("{}",inv.url())).await?;
//     guild.leave(ctx).await?;
//     Ok(())
// }