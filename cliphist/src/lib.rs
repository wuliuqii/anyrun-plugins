use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use utils::fuzzy_match;

#[derive(Deserialize)]
struct Config {
    max_entries: usize,
    cliphist_path: String,
    prefix: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_entries: 10,
            cliphist_path: "cliphist".into(),
            prefix: ":v".into(),
        }
    }
}

struct CliphistItem {
    id: usize,
    cliphist_id: String,
    content: String,
}

impl AsRef<str> for CliphistItem {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

struct State {
    config: Config,
    history: Vec<CliphistItem>,
}

#[init]
fn init(config_dir: RString) -> State {
    let config = match fs::read_to_string(format!("{}/cliphist.ron", config_dir)) {
        Ok(content) => ron::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    };

    let list_child = Command::new(&config.cliphist_path)
        .args(["list"])
        .output()
        .expect("Failed to execute cliphist list command");
    let list = String::from_utf8_lossy(&list_child.stdout).into_owned();

    let history = list
        .split('\n')
        .filter_map(|l| l.split_once('\t'))
        .enumerate()
        .map(|(id, (a, b))| CliphistItem {
            id,
            cliphist_id: a.to_string(),
            content: b.to_string(),
        })
        .collect::<Vec<_>>();

    State { config, history }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "cliphist".into(),
        icon: "view-list-symbolic".into(), // Icon from the icon theme
    }
}

#[get_matches]
fn get_matches(input: RString, state: &State) -> RVec<Match> {
    if !input.starts_with(&state.config.prefix) {
        return RVec::new();
    }

    let cleaned_input = &input[state.config.prefix.len()..];
    if cleaned_input.is_empty() {
        let entries = &state.history[..state.config.max_entries];
        entries
            .iter()
            .map(|item| {
                let title = item.content.clone();
                Match {
                    title: title.into(),
                    description: ROption::RNone,
                    use_pango: false,
                    icon: ROption::RNone,
                    id: ROption::RSome(item.id as u64),
                }
            })
            .collect()
    } else {
        let mut entries = fuzzy_match(cleaned_input, &state.history);
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(state.config.max_entries);
        entries
            .into_iter()
            .map(|(item, _)| {
                let title = item.content.clone();
                Match {
                    title: title.into(),
                    description: ROption::RNone,
                    use_pango: false,
                    icon: ROption::RNone,
                    id: ROption::RSome(item.id as u64),
                }
            })
            .collect()
    }
}

#[handler]
fn handler(selection: Match, state: &State) -> HandleResult {
    let id = state
        .history
        .iter()
        .find_map(|item| {
            if item.id as u64 == selection.id.unwrap() {
                Some(item.cliphist_id.clone())
            } else {
                None
            }
        })
        .map(|id| format!("{}\t ", id))
        .unwrap();

    let mut decode_child = Command::new(&state.config.cliphist_path)
        .args(["decode"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to decode cliphist entry");

    let mut decode_stdin = decode_child.stdin.take().expect("failed to get stdin");
    std::thread::spawn(move || {
        decode_stdin
            .write_all(id.as_bytes())
            .expect("failed to write to stdin")
    });

    let decode_out = decode_child
        .stdout
        .expect("Failed to spawn cliphist decode");

    let _copy_child = Command::new("wl-copy")
        .stdin(Stdio::from(decode_out))
        .spawn()
        .expect("Failed to spawn wl-copy");

    HandleResult::Close
}
