//! Provides the [`EventListenerPlugin`].

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use crate::{event_dispatcher::EventDispatcher, event_listener::EntityEvent};

/// The [`SystemSet`] that event listener plugins are added to.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct EventListenerSet;

/// Adds event listening and bubbling support for event `E`.
pub struct EventListenerPlugin<E: EntityEvent>(std::marker::PhantomData<E>);

impl<E: EntityEvent> Default for EventListenerPlugin<E> {
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
                    EventDispatcher::<E>::build,
                    EventDispatcher::<E>::bubble_events,
                    EventDispatcher::<E>::cleanup,
                )
                    .chain()
                    .run_if(on_event::<E>())
                    .in_set(EventListenerSet),
            );
    }
}
