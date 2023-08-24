//! Core implementation of event listening, bubbling, and callbacks.

use bevy::prelude::*;

use event_dispatcher::EventDispatcher;
use event_listener::EntityEvent;

pub mod callbacks;
pub mod event_dispatcher;
pub mod event_listener;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct EventListenerSet;

/// Adds event listening and bubbling support for event `E`.
pub struct EventListenerPlugin<E>(std::marker::PhantomData<E>);

impl<E> Default for EventListenerPlugin<E> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<E: EntityEvent> Plugin for EventListenerPlugin<E> {
    fn build(&self, app: &mut App) {
        app.add_event::<E>()
            .insert_resource(EventDispatcher::<E>::default())
            .add_systems(
                PreUpdate,
                (
                    EventDispatcher::<E>::build.run_if(on_event::<E>()),
                    EventDispatcher::<E>::bubble_events.run_if(on_event::<E>()),
                    EventDispatcher::<E>::cleanup.run_if(on_event::<E>()),
                )
                    .chain()
                    .in_set(EventListenerSet),
            );
    }
}
