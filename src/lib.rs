#![deny(missing_docs)]
//! Event listening, bubbling, and callbacks.

pub use bevy_eventlistener_core::*;
pub use bevy_eventlistener_derive::EntityEvent;

/// Common exports
pub mod prelude {
    pub use bevy_eventlistener_core::{
        callbacks::{Listener, ListenerMut},
        event_listener::{EntityEvent, On},
        EventListenerPlugin,
    };
    pub use bevy_eventlistener_derive::EntityEvent;
}
