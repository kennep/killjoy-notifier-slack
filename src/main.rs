use std::os::unix::process;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;

use structopt::StructOpt;
use anyhow::{Result, anyhow, Error, Context as AnyhowContext};

use dbus::blocking::Connection;
use dbus_crossroads::{Crossroads, MethodErr};

use xdg::BaseDirectories;

use serde::{Serialize, Deserialize};

use log::{debug, info};
use env_logger::Env;

const KILLJOY_BUS_NAME: &str = "com.wangpedersen.KilljoyNotifierSlack1";
const KILLJOY_OBJECT_PATH: &str = "/com/wangpedersen/KilljoyNotifierSlack1";

const KILLJOY_INTERFACE: &str = "name.jerebear.KilljoyNotifier1";
const KILLJOY_METHOD: &str = "Notify";

#[derive(Debug, StructOpt)]
#[structopt(name = "killjoy-notifier-slack", about = "Send Killjoy notifications to Slack")]
struct Cli {
    #[structopt(long)]
    system: bool,
    #[structopt(long)]
    user: bool
}

#[derive(Clone, Debug, Deserialize)]
struct Config {
    pub webhook_url: String,
    pub username: Option<String>,
    pub channel: Option<String>,
    pub icon_emoji: Option<String>
}

#[derive(Clone, Debug, Serialize)]
struct SlackAttachment {
    pub title: Option<String>,
    pub text: String,
    pub mrkdwn_in: Vec<String>
}

#[derive(Clone, Debug, Serialize)]
struct SlackPayload {
    pub attachments: Vec<SlackAttachment>,
    pub channel: Option<String>,
    pub username: Option<String>,
    pub icon_emoji: String
}

const DEFAULT_EMOJI: &str = ":robot_face:";

fn main() -> Result<()> {
    let args = Cli::from_args();

    env_logger::Builder::from_env(Env::default()
        .filter_or("KILLJOY_NOTIFIER_SLACK_LOG", "info")
        .write_style("KILLJOY_NOTIFIER_SLACK_LOG_STYLE")
    ).init();

    let path = get_load_path()?;

    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)
        .with_context(|| format!("Config file not valid: {}", path.display()))?;

    info!("Config file: {}", path.display());
    debug!("Webhook URL: {}", &config.webhook_url);
    debug!("Channel: {}", config.channel.as_ref().unwrap_or(&"(default)".to_string()));
    debug!("Username: {}", config.username.as_ref().unwrap_or(&"(default)".to_string()));
    debug!("Emoji: {}", config.icon_emoji.as_ref().unwrap_or(&DEFAULT_EMOJI.to_string()));

    let connection = match (process::parent_id(), args.system, args.user) {
        (_, true, true) => Err(anyhow!("--system and --user can not be used simultaneously")),
        (_, true, false) | (1, false, false) => Connection::new_system().map_err(Error::new),
        (_, false, _)  => Connection::new_session().map_err(Error::new),
    }?;

    let cr = register_object_path(config.clone())?;
    register_bus_name(&connection)?;
    cr.serve(&connection)?;

    Ok(())
}

fn register_bus_name(connection: &Connection) -> Result<()> {
    connection
        .request_name(KILLJOY_BUS_NAME, true, false, false)?;
    Ok(())
}

fn register_object_path(config: Config) -> Result<Crossroads> {
    let mut cr = Crossroads::new();

    let iface_token = cr.register(KILLJOY_INTERFACE, |interface| {
        interface.method(KILLJOY_METHOD, ("timestamp","unit name", "active states"), (), 
            |_context, config, args: (u64, String, Vec<String>)| {
                let (timestamp, unit_name, active_states) = args;
                info!("Notify: {0} {1} {2}", timestamp, unit_name, active_states.join(", "));
                post_slack_webhook(config, timestamp, &unit_name, &active_states)
                    .map_err(|e| MethodErr::failed(&e))            
            });
    });
    cr.insert(KILLJOY_OBJECT_PATH, &[iface_token], config );
    Ok(cr)
}

fn post_slack_webhook(config: &Config, _timestamp: u64, unit_name: &str, active_states: &Vec<String>) -> Result<()> {
    let client = reqwest::Client::new();

    let active_states_string = format_active_states(&active_states)?;
    let payload = SlackPayload {
        attachments: vec![SlackAttachment{
            title: Some(format!("{}: {}", unit_name, active_states_string)),
            text: format!("*{}* has transitioned to state: *{}*", unit_name, active_states.first().unwrap()).to_string(),
            mrkdwn_in: vec!["text".to_string()]
        }],
        channel: config.channel.clone(),
        username: config.username.clone(),
        icon_emoji: config.icon_emoji.clone().unwrap_or(":robot_face:".to_string())        
    };
    client.post(&config.webhook_url)
        .json(&payload)
        .send()?
        .error_for_status()
        .map(|_| ())
        .with_context(|| format!("Could not send request to webhook URL: {}", config.webhook_url))
}

fn get_load_path() -> Result<PathBuf> {
    let prefix = "killjoy";
    let suffix = "slack-notifier.json";
    BaseDirectories::with_prefix(prefix)
        .with_context(|| format!("Configuration file not found: {}/{}", prefix, suffix))?
        .find_config_file(suffix)
        .with_context(|| format!("Configuration file not found: {}/{}", prefix, suffix))
}

fn format_active_states(active_states: &Vec<String>) -> Result<String> {
    let formatted: String = active_states
        .chunks(2)
        .next()
        .ok_or(anyhow!("No active states given!"))?
        .iter()
        .rev()
        .map(|active_state: &String| -> &str { &**active_state })
        .collect::<Vec<&str>>()
        .join(" → ");
    Ok(formatted)
}