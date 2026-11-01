use kawi_plugin_sdk::exports::kawi::plugin::lifecycle::Guest;
use kawi_plugin_sdk::types::{Argument, Error, Event, SettingValue, State};
use kawi_plugin_sdk::{
    config_read, editor_decorations, editor_read, editor_signs, ui_message, ui_statusline,
    INVALID_PARAMS, STALE_REVISION,
};

mod markers;
mod visual;

use markers::{Counts, Marker};

const PROTOCOL_VERSION: u32 = 1;
const COMMAND_ID: &str = "scan";
const SHOW_SIGNS: &str = "show_signs";
const SHOW_VIRTUAL_TEXT: &str = "show_virtual_text";
const STALE_RETRY_LIMIT: usize = 2;

struct Plugin;

impl Guest for Plugin {
    fn init(host_protocol: u32) -> Result<(), Error> {
        if host_protocol == PROTOCOL_VERSION {
            Ok(())
        } else {
            Err(plugin_error(
                INVALID_PARAMS,
                "unsupported host protocol version",
            ))
        }
    }

    fn invoke_command(id: String, _args: Vec<Argument>) -> Result<Argument, Error> {
        if id != COMMAND_ID {
            return Err(plugin_error(INVALID_PARAMS, "unknown command"));
        }
        match refresh() {
            Ok(counts) => {
                let message = counts.message();
                ui_message::message(&message)?;
                Ok(Argument::Text(message))
            }
            Err(error) => {
                let _ = ui_message::message(&format!("markers: {}", error.message));
                Err(error)
            }
        }
    }

    fn on_event(event: Event) {
        if matches!(event, Event::DiagnosticsChanged(_)) {
            return;
        }
        if let Err(error) = refresh() {
            eprintln!("markers: {}", error.message);
        }
    }
}

fn refresh() -> Result<Counts, Error> {
    let mut last = None;
    for _ in 0..STALE_RETRY_LIMIT {
        match refresh_once() {
            Ok(counts) => return Ok(counts),
            Err(error) if error.code == STALE_REVISION => last = Some(error),
            Err(error) => return Err(error),
        }
    }
    Err(last.unwrap_or_else(|| plugin_error(STALE_REVISION, "stale revision")))
}

fn refresh_once() -> Result<Counts, Error> {
    let state = editor_read::get_state(true)?;
    let text = state.text.as_deref().unwrap_or_default();
    let markers = markers::scan(text);
    let counts = Counts::from_markers(&markers);
    publish(&state, &markers, &counts)?;
    Ok(counts)
}

fn publish(state: &State, markers: &[Marker], counts: &Counts) -> Result<(), Error> {
    let decorations = visual::decorations(markers, counts, setting_bool(SHOW_VIRTUAL_TEXT, true));
    editor_decorations::set(state.buffer, state.revision, &decorations)?;
    let signs = visual::signs(markers, setting_bool(SHOW_SIGNS, true));
    editor_signs::set(state.buffer, state.revision, &signs)?;
    let statusline = visual::statusline(counts);
    ui_statusline::set(&statusline)?;
    Ok(())
}

fn setting_bool(key: &str, default: bool) -> bool {
    match config_read::get(key) {
        Ok(Some(SettingValue::Boolean(value))) => value,
        Ok(_) => default,
        Err(error) => {
            eprintln!("markers: cannot read setting {key}: {}", error.message);
            default
        }
    }
}

fn plugin_error(code: i32, message: &str) -> Error {
    Error {
        code,
        message: message.to_string(),
        capability: None,
    }
}

kawi_plugin_sdk::export!(Plugin);
