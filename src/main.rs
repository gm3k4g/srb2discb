

/*
const UNDERLINE: &str = "\x1b[4m";
const UNDERLINE_RESET: &str = "\x1b[24m";
const BOLD_RESET: &str = "\x1b[22m";
const BOLD: &str = "\x1b[1m";


const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
*/

// TODO: Move this out of here ASAP
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

// how often to update messages on discord (in miliseconds)
const REFRESH_RATE: u64 = 1200;

use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

use tokio::time::Duration;

use serenity::http::Http;
use serenity::builder::{CreateEmbed, ExecuteWebhook};
use serenity::model::webhook::Webhook;
use serenity::model::prelude::Message;
use serenity::async_trait;
use serenity::client::{
    Context,EventHandler,Client
};
use serenity::model::gateway::Ready;
use serenity::all::GatewayIntents;


struct Handler;

// TODO: Error out if the lua file isn't detected on the SRB2 server?
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
    if msg.content == "!hello" {
        if let Err(e) = msg.channel_id.say(&ctx.http, "Hello, world!").await {
            println!("Error sending message: {:?}", e);
            }
        }
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
        // you may want to fix this.
        _ => {

            // Let's match arguments.
            // TODO: allow for argument stacking, except for the -h/-v directives. These will immediately print either help or version depending on which one was specified first.
            // Afterwards, the program will immediately quit.
            match args[1].as_str() {

                ARG_HELP_S | ARG_HELP => print_help(),
                ARG_VERSION_S | ARG_VERSION => print_version(),

                // starts by reading a provided config path. otherwise reads from a default config path.
                // TODO: could default config path be ~/.srb2/srb2discb.json?
                ARG_CONFIG_S | ARG_CONFIG => {  
                }, 

                // (start by) directly reading arguments provided via cli
                ARG_PARAMS_S | ARG_PARAMS => {
                    connect_bot().await;
                },
                _ => {},
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
        Err(_) => String::from(">>LOG FILE RELOAD<<"),
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
// Gets the range from `start` to `end` of a file.
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
           result.push_str(&line);
           result.push('\n');
       } else if line_number > end {
           break;
       }
   }

   Ok(result)
}

async fn connect_bot() {

    let data = fs::read_to_string("./secret.json")
        .expect("'secret.json' doesn't exist!");

    let json: serde_json::Value = serde_json::from_str(&data)
        .expect("JSON does not have correct format.");

    let token = json["bot_token"].as_str().unwrap();
    let url = json["log_channel"].as_str().unwrap();

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

    let msg_path = format!("{}{}", home_str, "/.srb2/luafiles/client/DiscordBot/Messages.txt");

    // TODO: log rotation yay or ney?
    // Replace messages, since there will be new ones ... if you want more details just check your rotating srb2 logs lol
    std::fs::write(&msg_path, "\n").unwrap();
    //let mut file = File::create(&msg_path).unwrap();

    //let mut prev_str = String::from("");\

    let mut seek_start = 0;

    //let mut str_arr : Vec<String> = Vec::new();

    // TODO: consider checking whether we're in log mode (reading directly from latest-log.txt) 
    //      or in srb2lua-log mode (reading directly from a file supplied by a lua script).
    //          In the former's case, read indiscriminately, [TODO!] we have not yet f
    //              made line reading consider the string limit, meaning that if the 
    //              seeker start and end are too far apart yet there's too many strings to
    //              concat, if the limit is reached the function returns, meaning that all the othe rstrings
    //              will be ignored as the starting seeker goes to the position of the ending seeker
    //              one SOLUTION to this is to tell the starting seeker to jump at the position of the last
    //              string we've read and continue reading.
    //          In the latter, it should be fine, but we should implement the above.
    loop {
        let seek_end = get_lines_num(&msg_path).unwrap();

          if seek_start != seek_end {
            let collected_strs = match read_range(&msg_path, seek_start, seek_end) {
                Ok(strs) => strs,
                Err(_) => continue,
            };
            if collected_strs.len() <= 1 {
                continue
            }
            //println!("{}", collected_strs);
            let webhook = Webhook::from_url(&http, url).await;
            let builder = ExecuteWebhook::new().content(collected_strs);
            webhook
                .expect("Couldn't run webhook")
                .execute(&http, false, builder).await.unwrap();

            seek_start = seek_end;
          }

        // Refresh period
        std::thread::sleep(Duration::from_millis(REFRESH_RATE));
    }
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
\t{NAME} {ARG_PARAMS_S}, {ARG_PARAMS} [CLI ARGS]
\t{NAME} {ARG_HELP_S}, {ARG_HELP}
\t{NAME} {ARG_VERSION_S}, {ARG_VERSION}
    
ARGS:
\t<CLI ARGS>
\t\tA list of arguments that SRB2 can process before the game starts. This can be one argument or many. It's advised to visit the SRB2 wiki ({CLI}) to look at what commandline arguments you can provide to SRB2.

\t\tFor example, to start a dedicated server on port 5029 and have your server show up in the standard rooms, execute the program like so:
\t\t  `{NAME} {ARG_PARAMS} -dedicated -port 5029 -room 38`


OPTIONS:
\t{ARG_PARAMS_S}, {ARG_PARAMS}
\t\tThis option is used to indicate that everything following this option will be parameters to give to SRB2 before it starts.

\t{ARG_HELP_S}, {ARG_HELP}
\t\tShow the help message.

\t{ARG_VERSION_S}, {ARG_VERSION}
\t\tShow program version.
");

}
