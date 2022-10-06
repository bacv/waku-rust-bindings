// std
use std::ffi::{c_char, CStr};
use std::ops::Deref;
use std::sync::RwLock;
// crates
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
// internal
use crate::general::{PubsubTopic, WakuMessage};

#[derive(Serialize, Deserialize)]
pub struct Signal {
    #[serde(alias = "type")]
    _type: String,
    event: Event,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "untagged", rename_all = "camelCase")]
pub enum Event {
    WakuMessage(WakuMessageEvent),
}

/// Type of `event` field for a `message` event
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WakuMessageEvent {
    /// The pubsub topic on which the message was received
    pubsub_topic: PubsubTopic,
    /// The message id
    message_id: String,
    /// The message in [`WakuMessage`] format
    waku_message: WakuMessage,
}

impl WakuMessageEvent {
    pub fn pubsub_topic(&self) -> &PubsubTopic {
        &self.pubsub_topic
    }

    pub fn message_id(&self) -> &String {
        &self.message_id
    }

    pub fn waku_message(&self) -> &WakuMessage {
        &self.waku_message
    }
}

/// Shared callback slot. Callbacks are registered here so they can be accessed by the extern "C"
#[allow(clippy::type_complexity)]
static CALLBACK: Lazy<RwLock<Box<dyn FnMut(Signal) + Send + Sync>>> =
    Lazy::new(|| RwLock::new(Box::new(|_| {})));

/// Register global callback
fn set_callback<F: FnMut(Signal) + Send + Sync + 'static>(f: F) {
    *CALLBACK.write().unwrap() = Box::new(f);
}

/// Wrapper callback, it transformst the `*const c_char` into a [`Signal`]
/// and executes the [`CALLBACK`] funtion with it
extern "C" fn callback(data: *const c_char) {
    let raw_response = unsafe { CStr::from_ptr(data) }
        .to_str()
        .expect("Not null ptr");
    let data: Signal = serde_json::from_str(raw_response).expect("Parsing signal to succeed");
    (CALLBACK
        .deref()
        .write()
        .expect("Access to the shared callback")
        .as_mut())(data)
}

/// Register callback to act as event handler and receive application signals,
/// which are used to react to asynchronous events in Waku
pub fn waku_set_event_callback<F: FnMut(Signal) + Send + Sync + 'static>(f: F) {
    set_callback(f);
    unsafe { waku_sys::waku_set_event_callback(&mut callback as *mut _ as *mut std::ffi::c_void) };
}

#[cfg(test)]
mod tests {
    use crate::events::waku_set_event_callback;

    // TODO: how to actually send a signal and check if the callback is run?
    #[test]
    fn set_event_callback() {
        waku_set_event_callback(|_signal| {});
    }
}