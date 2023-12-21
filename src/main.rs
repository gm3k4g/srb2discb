/*
const UNDERLINE: &str = "\x1b[4m";
const UNDERLINE_RESET: &str = "\x1b[24m";
const BOLD_RESET: &str = "\x1b[22m";
const BOLD: &str = "\x1b[1m";


const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
*/

const GREEN: &str = "\x1b[32m";
const WHITE: &str = "\x1b[0m";

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const CLI: &str = "https://wiki.srb2.org/wiki/Command_line_parameters";

// commandline argument names
const ARG_HELP: &str = "--help";
const ARG_HELP_S: &str = "-h";

const ARG_VERSION: &str = "--version";
const ARG_VERSION_S: &str = "-v";

const ARG_CONFIG: &str = "--config";
const ARG_CONFIG_S: &str = "-c";

const ARG_PARAMS: &str = "--params";
const ARG_PARAMS_S: &str = "-p";

const ARG_RUN: &str = "--run";
const ARG_RUN_S: &str = "-r";

// how often to update messages on discord (in miliseconds)
const REFRESH_RATE: u64 = 1200;

// TODO: use config variable instead of `.srb2`
const _LATEST_LOG: &str = "/.srb2/latest-log.txt";
const MESSAGES_TXT: &str = "/.srb2/luafiles/client/DiscordBot/Messages.txt";
const DISCMSG_TXT: &str = "/.srb2/luafiles/client/DiscordBot/discordmessage.txt";

use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::fs::OpenOptions;

use tokio::time::Duration;

use serenity::http::Http;
use serenity::builder::CreateMessage;
use serenity::builder::CreateAllowedMentions as Am;
use serenity::async_trait;
use serenity::client::{
    Context,EventHandler,Client
};
use serenity::model::gateway::Ready;
use serenity::all::GatewayIntents;
use serenity::framework::standard::StandardFramework;

use serenity::model::id::ChannelId;
use serenity::model::id::GuildId;
use serenity::model::channel::Message;

//
// Removes any potential trash/unicode characters in the string.
// The string itself adheres to ASCII only, staying between the ranges 20 and 7F (hex) in the ASCII table.
// Afterwards, returns an entirely new String whose contents have been purged properly.
//
async fn discord_to_srb2(ctx: &Context, msg: Message) -> String {
    let mut copy = String::from(msg.content.as_str());
    
    copy.retain(|c| {
        let ascii = c as u32;
        ascii >= 32 && ascii <= 126
    });

    // Replace backward slashes.
    copy = copy.replace("\n", "\\n");

    // Let's look for any potential links, and get rid of them.
    let copy_splitted: Vec<&str> = copy.split_whitespace().collect();
    let copy_only_links: Vec<&str> = copy_splitted
        .into_iter()
        .filter(|word| {
            word.contains("https://") || word.contains("http://")
            })
        .collect();
    
    // Let's remove links
    let mut no_links = copy.clone();
    for link in &copy_only_links {
        no_links = no_links.replace(link, "[LINK]");
    }

    // Turn mentions into names
    let mut mentions = no_links.clone();
    let guild_id = msg.guild_id.unwrap();

    for mention in &msg.mentions {
        let name = match mention.nick_in(ctx, guild_id).await {
            Some(nick) => nick.to_string(),
            None => {
                match mention.global_name.clone() {
                    Some(name) => name,
                    None => mention.name.clone()
                }
            }
        };

        mentions = mentions.replace(format!("<@{}>", mention.id.get()).as_str(), &name);
    };

    // The result.
    let result = mentions;
    result
}

struct Handler;

// TODO: Error out if the lua file isn't detected on the SRB2 server?
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {

        // Get channel from json
        let data = fs::read_to_string("./secret.json")
            .expect("'secret.json' doesn't exist!");
        let json: serde_json::Value = serde_json::from_str(&data)
            .expect("JSON file doesn't have a correct format!");
        // ok but this is a bruh moment. why i gotta make them strs first and then u64s to make it not panic?
        let json_channel_id = json["channel_id"].as_str().unwrap();
        let json_bot_id = json["bot_id"].as_str().unwrap();

        let channel_id = format!("{}",msg.channel_id.get());
        let bot_id = msg.author.id.get();


        // TODO: privacy concern; if people fuck around with the IDs, they could infiltrate foreign channels. any way to avoid this?
        // Any messages in this channel are sent to the SRB2 server
        if channel_id == json_channel_id && bot_id != json_bot_id.parse::<u64>().unwrap()
        {
            // Access messages
            let home = home::home_dir().unwrap();
            let home_str = home.to_str().unwrap();

            let discord_msg_path = format!("{}{}", home_str, DISCMSG_TXT);
            let server_msg_path  = format!("{}{}", home_str, MESSAGES_TXT);

            // Open the discord messages file
            let disc_file = match OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .create_new(false)
                .append(true)
                .open(discord_msg_path) {
                    Ok(f) => f,
                    Err(e) => {
                        println!("ERROR: {}", e);
                        println!("TODO: implement file creation if nonexistent.");
                        return;
                    },
                };

            // Also open the messages file
            let _msg_file = match OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .create_new(false)
                .append(true)
                .open(server_msg_path) {
                    Ok(f) => f,
                    Err(e) => {
                        println!("ERROR: {}", e);
                        println!("TODO: implement file creation if nonexistent.");
                        return;
                    },
                };

            // Grab user name. If they have a server-specific name, use that, else use their global name, else use their account name.
            let user_name = match msg.member.clone().unwrap().nick {
                Some(name) => name,
                None => match msg.author.global_name {
                    Some(ref name) => name,
                    None => &msg.author.name,
                }.to_string() , //HUH??
            };

            // Write message in the discord message text file of SRB2
            let msg_content = discord_to_srb2(&ctx, msg).await;
            if msg_content.len() > 1 {
                let _ = writeln!(&disc_file, "{}", format!("<{}> {}", user_name, msg_content));    
            }
            //let _ = writeln!(&msg_file , "{}", format!("[Discord]<{}> {}", user_name, msg.content));
        }

        /*
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "Hello, world!").await {
                println!("Error sending message: {:?}", e);
            }
        }*/
    }

    async fn ready(&self, _: Context, _ready: Ready) {
        println!(
"
{GREEN}===============================================================================
\tThe SRB2 discord bot has been connected and is now active!
\tMake sure that you've added the lua file to your SRB2 server!
==============================================================================={WHITE}"
        );
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Print help if specified args are less than 1.
    match args.len() {
        1 => {
            print_help();
        },

        // Otherwise, analyze args.
        // TODO: This 'analysis' simply takes one argument and reads the rest as input for that arg.
        // What you want is to instead stack arguments (e.g. -p blah blah -)
        // basically, allow for argument stacking, except for the -h/-v directives. These will immediately print either help or version depending on which one was specified first.
        _ => {

            // Let's match arguments.

            // Afterwards, the program will immediately quit.
            match args[1].as_str() {

                ARG_HELP_S | ARG_HELP => print_help(),
                ARG_VERSION_S | ARG_VERSION => print_version(),

                // (start by) directly reading arguments provided via cli
                ARG_PARAMS_S | ARG_PARAMS => {
                    todo!()
                }

                // starts by reading a provided config path. otherwise reads from a default config path.
                // TODO: could default config path be ~/.srb2/srb2discb.cfg?
                ARG_CONFIG_S | ARG_CONFIG => {  
                    todo!()
                }, 

                ARG_RUN_S | ARG_RUN => {
                    connect_bot().await;
                },
                _ => print_help(), //print help just in case
            }

        }
    }
}

//
// Get the total lines of the file.
//
fn get_lines_num(file_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let line = match std::fs::read_to_string(file_path) {
        Ok(line) => line,
        Err(e) => {
            println!("ERROR: {}", e);
            String::from("-")    
        },
    };
        
    Ok(line.lines().count())
}

//
// Get the last line of the file.
//
/*
fn get_last_line(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {

    let line = match std::fs::read_to_string(file_path) {
        Ok(line) => line,
        Err(err) => String::from(""), //"LOG FILE RELOADING"),
    };
        
    let line = line.lines().collect::<Vec<_>>();

    Ok(line[line.len().saturating_sub(1)..line.len()]
        .iter()
        .map(|&s| String::from(s)).collect())
}
*/

//
// Gets the last match of a specified string from a file.
// Returns a Result containing a String.
//
fn _get_last_match(filename: &str, target: &str) -> Option<String> {
   let file = File::open(filename).ok()?;
   let reader = BufReader::new(file);
   let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().ok()?;
   let reversed_lines: Vec<&str> = lines.iter().rev().map(AsRef::as_ref).collect();

   match reversed_lines.iter().position(|&line| line.contains(target)) {
       Some(index) => Some(lines[index].clone()),
       None => None,
   }
}

//
// Gets the range from `start` to `end` of a file. 
// Returns a Result containing a String.
//
fn read_range(file_path: &str, start: usize, end: usize) -> Result<String, Box<dyn std::error::Error>> {
   let file = File::open(file_path)?;
   let reader = BufReader::new(file);

   let mut result = String::new();
   let mut line_number = 0;

   for line in reader.lines() {
       let string = line.unwrap();
       line_number += 1;

       // TODO: If the seek start and seek end postiions are too far apart, some things will be missed
       // How to fix it?
       // Assumption for now is that we will be only receiving limitd amount of messages so it won't hurt a lot.

       // Normally 1059 but just to be safe, 1000 ig
       if result.len() > 1000 {
        break;
       }
       
       if line_number > start && line_number <= end {
           let line = string;
           /* TODO: Detect any kick/bans. Get the server's log file. Search for the last mention of 'ban/kick' and get the reason
           and then make it part of the message to relay to discord.
           // If there's any kicks/bans, grab the reason message directly from the server's log file.
           if line.contains("kick") {
                let home = home::home_dir().unwrap();
                let home_str = home.to_str().unwrap();
                let log_path = format!("{}{}", home_str, LATEST_LOG);

                // This can be none if you somehow clobber `latest-log.txt` with another srb2 executable.
                // Be careful about this.
                let line_reason = match get_last_match(&log_path, "kick") {
                    Some(string) => string,
                    None => String::from("None"),
                };
                println!("{line_reason}");
           }
           */

           result.push_str(&line);
           result.push('\n');
       } else if line_number > end {
           break;
       }
   }

   Ok(result)
}

//
// Any mentions of server emojis are replaced with emojis properly.
// In addition, any special symbols that may invoke discord markdown (*, _, `, etc.)
// get formatted so they don't invoke discord markdown (e.g. by appending a backward slash \ behind them.)
//
async fn replace_emojis(id: u64, http: &Http, content: &str) -> String {
   let mut new_content: String = String::from(content);
   let guild_id = GuildId::new(id);
   let emojis = http.get_emojis(guild_id).await.unwrap();

   // Go over all emojis of the server and replace with actual emojis, if they're occurring in the string.
   for emoji in &emojis {
        let server_emoji = format!(":{}:", emoji.name);
        new_content = new_content.replace(&server_emoji, format!("<:{}:{}>", emoji.name, emoji.id.get()).as_str());
   }

   new_content
}


async fn connect_bot() {
    let data = fs::read_to_string("./secret.json")
        .expect("'secret.json' doesn't exist!");

    let json: serde_json::Value = serde_json::from_str(&data)
        .expect("JSON file doesn't have a correct format!");

    let token = json["bot_token"].as_str().unwrap();

    // ok but this is a bruh moment. why i gotta make them strs first and then u64s to make it not panic?
    let guild = json["guild_id"].as_str().unwrap();
    let channel = json["channel_id"].as_str().unwrap();

    let chnl_id = channel.parse().unwrap();
    let guild_id = guild.parse().unwrap();

    let http = Http::new(token);

    // TODO: turn this into cli arg (?)
    let home = home::home_dir().unwrap();
    let home_str = home.to_str().unwrap();
    /*match home::home_dir() {
        Some(path) => path.to_str().unwrap(),
        None => {
            println!("Impossible to get your home dir!");
            ""
        }
    };*/

    // TODO: create path if it doesn't exist
    // this was gonna be used to look up kick/ban message reasons.
    let disc_path = format!("{}{}", home_str, DISCMSG_TXT);
    let msg_path  = format!("{}{}", home_str, MESSAGES_TXT);

    // TODO: Turn this into a function
    // TODO: log rotation yay or ney?
    // Replace messages, since there will be new ones ... if you want more details just check your rotating srb2 logs lol
    match std::fs::write(&msg_path, "\n") {
        Ok(_) => {},

        // Create file if it's nonexistent
        Err(e) => {
            println!("ERROR: {e}");
            println!("Attempting to create file...");

            // Create the directory if it doesn't exist
            let path = std::path::Path::new(&msg_path);
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();

            // Open the discord messages file. Create a new one if it doesn't exist
            let _ = match OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .create_new(true)
                .append(true)
                .open(msg_path.clone()) {
                    Ok(f) => f,
                    Err(e) => {
                        // TODO: Unsure what the error here would be and how to handle it.
                        println!("ERROR: {}", e);
                        return;
                    },
                };
        }
    };

    // Do the same for the discord messages.
    // Replace messages, since there will be new ones ... if you want more details just check your rotating srb2 logs lol
    match std::fs::write(&disc_path, "\n") {
        Ok(_) => {},

        // Create file if it's nonexistent
        Err(e) => {
            println!("ERROR: {e}");
            println!("Attempting to create file...");

            // Create the directory if it doesn't exist
            let path = std::path::Path::new(&disc_path);
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();

            // Open the discord messages file. Create a new one if it doesn't exist
            let _ = match OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .create_new(true)
                .append(true)
                .open(disc_path.clone()) {
                    Ok(f) => f,
                    Err(e) => {
                        // TODO: Unsure what the error here would be and how to handle it.
                        println!("ERROR: {}", e);
                        return;
                    },
                };
        }
    };


    //let mut file = File::create(&msg_path).unwrap();
    let mut seek_start = 0;
    let framework = StandardFramework::new();

    // TODO: privacy concerns: Bot knows people's messages/DMs
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");


    // Relay chat loop start
    let loop_task = tokio::spawn(async move {
        loop {
            let seek_end = get_lines_num(&msg_path).unwrap();

            if seek_start != seek_end {
                let mut collected_strs = match read_range(&msg_path, seek_start, seek_end) {
                    Ok(strs) => strs,
                    Err(_) => continue,
                };

                if collected_strs.len() <= 1 {
                    continue
                } else {

                   collected_strs = replace_emojis(guild_id, &http, &collected_strs).await;
                }

                // Bot: You are not allowed to mention anyone/any role.
                let mentions = Am::new()
                    .all_users(false)
                    .all_roles(false)
                    .everyone(false);

                let message = format!("{collected_strs}");
                let builder = CreateMessage::new()
                    .content(message.clone())
                    .allowed_mentions(mentions);


                // Send a message to the Discord channel
                let channel_id = ChannelId::new(chnl_id); // Replace with your channel ID

                // Ignore errors
                let _msg = match channel_id.send_message(&http, builder).await {
                    Ok(_) => {},
                    Err(e) => {
                        println!("ERROR: {}", e);
                    },
                };

                seek_start = seek_end;
            }

            tokio::time::sleep(Duration::from_millis(REFRESH_RATE)).await;
        }
    });

    // Start the bot.. done here, because it's async i guess
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    // Wait for the loop task to finish
    let _ = loop_task.await;
}

async fn _login_bot() {
    let data = fs::read_to_string("./secret.json")
        .expect("'secret.json' doesn't exist!");

    let json: serde_json::Value = serde_json::from_str(&data)
        .expect("JSON does not have correct format.");

    let token = json["bot_token"].as_str().unwrap();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client =
        Client::builder(&token, intents)
            .event_handler(Handler)
            .await
            .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

fn print_version() {
    println!("{NAME} v{VERSION}");
}

fn print_help() {
    println!(
"{NAME} v{VERSION}
AUTHOR: {AUTHOR}

DESCRIPTION: {DESCRIPTION}

To print this message again, you can either call {NAME} with no arguments, or call it with the `{ARG_HELP_S}` or `{ARG_HELP}` arguments.

USAGE:
\t{NAME} {ARG_RUN_S}, {ARG_RUN}
\t{NAME} {ARG_HELP_S}, {ARG_HELP}
\t{NAME} {ARG_VERSION_S}, {ARG_VERSION}

WIP:
\t{NAME} {ARG_RUN_S}, {ARG_RUN} [CLI ARGS]
\t{NAME} {ARG_PARAMS_S}, {ARG_PARAMS} (TODO)
\t{NAME} {ARG_CONFIG_S}, {ARG_CONFIG} (TODO)
    
ARGS:
\t<CLI ARGS>
\t\tA list of arguments that SRB2 can process before the game starts. This can be one argument or many. It's advised to visit the SRB2 wiki ({CLI}) to look at what commandline arguments you can provide to SRB2.

\t\tFor example, to start a dedicated server on port 5029 and have your server show up in the standard rooms, execute the program like so:
\t\t  `{NAME} {ARG_PARAMS} -dedicated -port 5029 -room 38`


OPTIONS:
\t{ARG_PARAMS_S}, {ARG_PARAMS}
\t\tThis option is used to indicate that everything following this option will be parameters to give to SRB2 before it starts.
\t\tNote that this on its own will not run the program, as `{ARG_RUN_S} | {ARG_RUN}` are required to do that.

\t{ARG_CONFIG_S}, {ARG_CONFIG}
\t\tIf specified on its own, it opens the config at the default path and directly reads the arguments from there. If one doesn't already exist, it will be created. 
\t\tIf you have a config at a specific path, then you may specify this path (e.g. -c /path/to/my/file.sh ).
\t\tNote that this on its own will not run the program, as `{ARG_RUN_S} | {ARG_RUN}` are required to do that.

\t{ARG_RUN_S}, {ARG_RUN}
\t\tIf specified on its own, immediately starts the program with the default parameters. That is to say, it will look for a default config and read for any potential arguments before starting.
\t\tThis is primarily stacked with the above parameters (e.g. config/parameters)
\t\tThere are some extra parameters you can provide here, and they are as follows:
\t\t\tNOCONFIG: This will tell the program to just start without reading from any default config. (e.g. --start NOCONFIG )
\t\t\tRUNSRB2 : This will tell the program to run the program, but to also run SRB2. You can have more control over SRB2 this way.
\t\t\t\tHere you can also specify a path to your SRB2 executable. (e.g. --start RUNSRB2 /path/to/srb2)

\t{ARG_HELP_S}, {ARG_HELP}
\t\tShow the help message.

\t{ARG_VERSION_S}, {ARG_VERSION}
\t\tShow program version.
");

}
