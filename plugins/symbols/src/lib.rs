use std::{collections::HashMap, fs};

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;
use utils::fuzzy_match;

include!(concat!(env!("OUT_DIR"), "/unicode.rs"));

#[derive(Clone, Debug)]
struct Symbol {
    chr: String,
    name: String,
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Debug)]
struct Config {
    prefix: String,
    symbols: HashMap<String, String>,
    max_entries: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: ":s".to_string(),
            symbols: HashMap::new(),
            max_entries: 3,
        }
    }
}

struct State {
    config: Config,
    symbols: Vec<Symbol>,
}

#[init]
fn init(config_dir: RString) -> State {
    // Try to load the config file, if it does not exist only use the static unicode characters
    let config = if let Ok(content) = fs::read_to_string(format!("{}/symbols.ron", config_dir)) {
        ron::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    };

    let symbols = UNICODE_CHARS
        .iter()
        .map(|(name, chr)| (name.to_string(), chr.to_string()))
        .chain(config.symbols.clone().into_iter())
        .map(|(name, chr)| Symbol { chr, name })
        .collect();

    State { config, symbols }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Symbols".into(),
        icon: "accessories-character-map".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &State) -> RVec<Match> {
    if !input.starts_with(&state.config.prefix) {
        return RVec::new();
    }

    let input = if let Some(input) = input.strip_prefix(&state.config.prefix) {
        input.trim()
    } else {
        return RVec::new();
    };

    let mut symbols = fuzzy_match(input, &state.symbols);
    symbols.sort_by(|a, b| b.1.cmp(&a.1));
    symbols.truncate(state.config.max_entries);
    symbols
        .into_iter()
        .map(|(symbol, _)| Match {
            title: symbol.chr.clone().into(),
            description: ROption::RSome(symbol.name.clone().into()),
            use_pango: false,
            icon: ROption::RNone,
            id: ROption::RNone,
        })
        .collect()
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    HandleResult::Copy(selection.title.into_bytes())
}
