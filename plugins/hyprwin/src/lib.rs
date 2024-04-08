use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use core::ops::Deref;
use hyprland::data::{Client, Clients};
use hyprland::shared::HyprData;
use serde::Deserialize;
use std::fs;
use utils::fuzzy_match;

#[derive(Debug, Clone)]
struct ClientId {
    client: Client,
    search: String,
    id: u64,
}

impl Deref for ClientId {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl AsRef<str> for ClientId {
    fn as_ref(&self) -> &str {
        &self.search
    }
}

#[derive(Deserialize)]
struct Config {
    max_entries: usize,
    prefix: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_entries: 5,
            prefix: "/".into(),
        }
    }
}

struct State {
    clients: Vec<ClientId>,
    config: Config,
}

#[init]
fn init(config_dir: RString) -> State {
    let config = match fs::read_to_string(format!("{}/hyprwin.ron", config_dir)) {
        Ok(content) => ron::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    };

    State {
        clients: Clients::get()
            .expect("Failed to get clients")
            .filter(|client| !(client.title.is_empty() && client.class.is_empty()))
            .enumerate()
            .map(|(idx, client)| ClientId {
                id: idx as u64,
                search: format!("{}: {}", client.class, client.title),
                client,
            })
            .collect(),
        config,
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Hyprland Windows".into(),
        icon: "window-new".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &State) -> RVec<Match> {
    let input = if let Some(input) = input.strip_prefix(&state.config.prefix) {
        input.trim()
    } else {
        return RVec::new();
    };

    let mut entries = fuzzy_match(input, &state.clients);
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.truncate(state.config.max_entries);
    entries
        .into_iter()
        .map(|(client, _)| Match {
            title: client.class.clone().into(),
            icon: ROption::RSome(icon_from_class(&client.class).into()),
            use_pango: false,
            description: ROption::RSome(client.title.clone().into()),
            id: ROption::RSome(client.id),
        })
        .collect::<Vec<Match>>()
        .into()
}

#[handler]
fn handler(selection: Match, state: &State) -> HandleResult {
    // Handle the selected match and return how anyrun should proceed
    use hyprland::dispatch::*;
    let Some(address) = state
        .clients
        .iter()
        .find(|c| c.id == selection.id.unwrap_or_default())
        .map(|c| c.address.clone())
    else {
        return HandleResult::Close;
    };
    Dispatch::call(DispatchType::FocusWindow(WindowIdentifier::Address(
        address,
    )))
    .expect("Unable to focus hyprland window");
    HandleResult::Close
}

fn icon_from_class(class: impl AsRef<str>) -> String {
    let class = class.as_ref().to_lowercase();
    if class.contains('.') {
        class.split('.').last().unwrap_or_default().into()
    } else {
        class
    }
}
