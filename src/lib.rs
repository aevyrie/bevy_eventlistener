//! Event listening, bubbling, and callbacks.

use bevy::prelude::*;

use listener_graph::EventDispatcher;
use on_event::EntityEvent;

pub mod prelude {}

pub mod callbacks;
pub mod listener_graph;
pub mod on_event;

/// Adds event listening and bubbling support for event `E`.
pub struct EventListenerPlugin<E>(std::marker::PhantomData<E>);

impl<E> Default for EventListenerPlugin<E> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<E: EntityEvent> Plugin for EventListenerPlugin<E> {
    fn build(&self, app: &mut App) {
        app.insert_resource(EventDispatcher::<E>::default())
            .add_systems(
                (
                    EventDispatcher::<E>::build.run_if(on_event::<E>()),
                    EventDispatcher::<E>::bubble_events.run_if(on_event::<E>()),
                    EventDispatcher::<E>::cleanup.run_if(on_event::<E>()),
                )
                    .chain(),
            );
    }
}
