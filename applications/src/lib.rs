use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::{anyrun_interface::HandleResult, *};
use scrubber::NucleoEntry;
use serde::Deserialize;
use std::{env, fs, process::Command};

use utils::fuzzy_match;

#[derive(Deserialize)]
pub struct Config {
    desktop_actions: bool,
    max_entries: usize,
    terminal: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            desktop_actions: false,
            max_entries: 5,
            terminal: Some("wezterm".into()),
        }
    }
}

pub struct State {
    config: Config,
    entries: Vec<NucleoEntry>,
}

mod scrubber;

const SENSIBLE_TERMINALS: &[&str] = &["alacritty", "foot", "kitty", "wezterm", "wterm"];

#[handler]
pub fn handler(selection: Match, state: &State) -> HandleResult {
    let entry = state
        .entries
        .iter()
        .find(|entry| entry.id == selection.id.unwrap())
        .unwrap();

    let desktop_entry = &entry.desktop_entry;

    if desktop_entry.term {
        match &state.config.terminal {
            Some(term) => {
                if let Err(why) = Command::new(term)
                    .arg("-e")
                    .arg(&desktop_entry.exec)
                    .spawn()
                {
                    eprintln!("Error running desktop entry: {}", why);
                }
            }
            None => {
                for term in SENSIBLE_TERMINALS {
                    if Command::new(term)
                        .arg("-e")
                        .arg(&desktop_entry.exec)
                        .spawn()
                        .is_ok()
                    {
                        break;
                    }
                }
            }
        }
    } else if let Err(why) = Command::new("sh")
        .arg("-c")
        .arg(&desktop_entry.exec)
        .current_dir(
            desktop_entry
                .path
                .as_ref()
                .unwrap_or(&env::current_dir().unwrap()),
        )
        .spawn()
    {
        eprintln!("Error running desktop entry: {}", why);
    }

    HandleResult::Close
}

#[init]
pub fn init(config_dir: RString) -> State {
    let config: Config = match fs::read_to_string(format!("{}/applications.ron", config_dir)) {
        Ok(content) => ron::from_str(&content).unwrap_or_else(|why| {
            eprintln!("Error parsing applications plugin config: {}", why);
            Config::default()
        }),
        Err(why) => {
            eprintln!("Error reading applications plugin config: {}", why);
            Config::default()
        }
    };

    let entries = scrubber::scrubber(&config).unwrap_or_else(|why| {
        eprintln!("Failed to load desktop entries: {}", why);
        Vec::new()
    });

    State { config, entries }
}

#[get_matches]
pub fn get_matches(input: RString, state: &State) -> RVec<Match> {
    let mut entries = fuzzy_match(&input, &state.entries);
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    entries.truncate(state.config.max_entries);
    entries
        .into_iter()
        .map(|(entry, _)| Match {
            title: entry.desktop_entry.name.clone().into(),
            description: entry
                .desktop_entry
                .desc
                .clone()
                .map(|desc| desc.into())
                .into(),
            use_pango: false,
            icon: ROption::RSome(entry.desktop_entry.icon.clone().into()),
            id: ROption::RSome(entry.id),
        })
        .collect()
}

#[info]
pub fn info() -> PluginInfo {
    PluginInfo {
        name: "Applications".into(),
        icon: "application-x-executable".into(),
    }
}
