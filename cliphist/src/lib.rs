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

#[derive(Debug)]
enum Error {
    CliphistCommandFailed(std::io::Error),
    CliphistReturnErrorCode(i32),
    Stdin,
    Thread,
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

    let output = Command::new(&config.cliphist_path)
        .args(["list"])
        .output()
        .map_err(Error::CliphistCommandFailed);

    let content = match output {
        Ok(o) => {
            if o.status.success() {
                Ok(String::from_utf8_lossy(&o.stdout).into_owned())
            } else {
                Err(Error::CliphistReturnErrorCode(o.status.code().unwrap_or(1)))
            }
        }
        Err(e) => Err(e),
    };

    let history = content.map(|s| {
        s.split('\n')
            .filter_map(|l| l.split_once('\t'))
            .enumerate()
            .map(|(id, (a, b))| CliphistItem {
                id,
                cliphist_id: a.to_string(),
                content: b.to_string(),
            })
            .collect::<Vec<_>>()
    });

    history
        .map(|history: Vec<CliphistItem>| State { config, history })
        .unwrap()
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

    let child = Command::new(&state.config.cliphist_path)
        .args(["decode"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(Error::CliphistCommandFailed);

    let output = child.and_then(|mut c| {
        let write_to_stdin = c.stdin.take().ok_or(Error::Stdin).and_then(|mut stdin| {
            std::thread::spawn(move || stdin.write_all(id.as_bytes()).map_err(|_| Error::Stdin))
                .join()
                .map_err(|_| Error::Thread)
                .and_then(|r| r)
        });
        write_to_stdin.and_then(|_| c.wait_with_output().map_err(Error::CliphistCommandFailed))
    });

    output
        .map(|bytes| HandleResult::Copy(bytes.stdout.into()))
        .unwrap()
}
