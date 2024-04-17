//! This module provides event listeners, [`On`], the most important part of
//! [`bevy_eventlistener`](crate).

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::callbacks::{CallbackSystem, ListenerInput};
use bevy_ecs::identifier;
use bevy_ecs::{
    prelude::*,
    system::{Command, EntityCommands},
};
#[cfg(feature = "trace")]
use bevy_utils::tracing::error;

/// An event that targets a specific entity, and should support event listeners and bubbling.
pub trait EntityEvent: Event + Clone {
    /// The entity that was targeted by this event, e.g. the entity that was clicked on.
    fn target(&self) -> Entity;
    /// Should events of this type bubble up the entity hierarchy, starting from the target? This is
    /// enabled by default.
    fn can_bubble(&self) -> bool {
        false
    }
}

/// `ListenerHandle` is a struct that generates unique identifiers for event listeners.
/// It uses an atomic counter to ensure that each ID is unique and that IDs can be generated safely in a multi-threaded context.
#[derive(Default)]
pub struct ListenerId {
    /// An atomic counter used to generate unique IDs for event listeners.
    id_counter: AtomicUsize,
}

impl ListenerId {
    /// Creates a new `ListenerHandle` with its internal counter set to 0.
    pub fn new() -> Self {
        Self {
            id_counter: AtomicUsize::new(0),
        }
    }

    /// Generates a new unique ID by incrementing the internal counter and returning its value.
    /// The `fetch_add` method ensures that this operation is atomic, so it's safe to call in a multi-threaded context.
    pub fn next_id(&self) -> usize {
        self.id_counter.fetch_add(1, Ordering::SeqCst)
    }
}

/// An event listener with a callback that is triggered when an [`EntityEvent`] bubbles past or
/// targets this entity.
///
/// The building block for all event listeners is the [`On::run`] method. All the other provided
/// methods are convenience methods that describe the most common functionality. However, because
/// these all use the public [`On::run`] method internally, you can easily define your own variants
/// that have the behavior you want!
#[derive(Component, Default)]
pub struct On<E: EntityEvent> {
    phantom: PhantomData<E>,
    listener_id: ListenerId,
    /// use tuple as we want to keep the order of the callbacks with hashmap pattern
    pub(crate) callbacks: Vec<(usize, CallbackSystem)>,
}

impl<E: EntityEvent> On<E> {
    /// Run a callback system every time this event listener is triggered. This can be a closure or
    /// a function, as described by bevy's documentation. The only notable difference from Bevy
    /// systems is that the callback system can access a resource with event data,
    /// [`ListenerInput`]. You can more easily access this with the system params
    /// [`Listener`](crate::callbacks::Listener) and [`ListenerMut`](crate::callbacks::ListenerMut).
    pub fn run<Marker>(&mut self, callback: impl IntoSystem<(), (), Marker>) -> usize {
        let id = self.listener_id.next_id();
        self.callbacks.push((id, CallbackSystem::New(Box::new(IntoSystem::into_system(callback)))));
        id
    }

    /// constructor for empty event listener
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            listener_id: ListenerId::new(),
            callbacks: Vec::new(),
        }
    }

    /// Remove a callback from this event listener.
    pub fn remove(&mut self, handle: ListenerHandle<E>) {
        self.callbacks.retain(|(h, _)| h.id != handle.id);
    }

    /// Replace a callback with a new one. This is useful for hot-reloading systems.
    pub fn replace<Marker>(&mut self, handle: ListenerHandle<E>, callback: impl IntoSystem<(), (), Marker>)
    {
        if let Some((_, cb)) = self.callbacks.iter_mut().find(|(h, _)| h.id == handle.id) {
            *cb = CallbackSystem::New(Box::new(IntoSystem::into_system(callback)));
        }
    }

    /// Add a single [`Command`] any time this event listener is triggered. The command must
    /// implement `From<E>`.
    pub fn add_command<C: From<ListenerInput<E>> + Command + Send + Sync + 'static>(&mut self) -> ListenerHandle<E> {
        self.run(|event: Res<ListenerInput<E>>, mut commands: Commands| {
            commands.add(C::from(event.to_owned()));
        })
    }

    /// Get mutable access to [`Commands`] any time this event listener is triggered.
    pub fn commands_mut(
        &mut self,
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut Commands),
    ) -> ListenerHandle<E> {
        self.run(
            move |mut event: ResMut<ListenerInput<E>>, mut commands: Commands| {
                func(&mut event, &mut commands);
            },
        )
    }

    /// Get mutable access to the target entity's [`EntityCommands`] using a closure any time this
    /// event listener is triggered.
    pub fn target_commands_mut(
        &mut self,
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut EntityCommands),
    ) -> ListenerHandle<E> {
        self.run(
            move |mut event: ResMut<ListenerInput<E>>, mut commands: Commands| {
                let target = event.target();
                func(&mut event, &mut commands.entity(target));
            },
        )
    }

    /// Insert a bundle on the target entity any time this event listener is triggered.
    pub fn target_insert(&mut self, bundle: impl Bundle + Clone) -> ListenerHandle<E> {
        self.run(
            move |event: Res<ListenerInput<E>>, mut commands: Commands| {
                let bundle = bundle.clone();
                commands.entity(event.target()).insert(bundle);
            },
        )
    }

    /// Remove a bundle from the target entity any time this event listener is triggered.
    pub fn target_remove<B: Bundle>(&mut self) -> ListenerHandle<E> {
        self.run(|event: Res<ListenerInput<E>>, mut commands: Commands| {
            commands.entity(event.target()).remove::<B>();
        })
    }

    /// Get mutable access to a specific component on the target entity using a closure any time
    /// this event listener is triggered. If the component does not exist, an error will be logged.
    pub fn target_component_mut<C: Component>(
        &mut self,
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut C),
    ) -> ListenerHandle<E> {
        self.run(
            move |mut event: ResMut<ListenerInput<E>>, mut query: Query<&mut C>| {
                if let Ok(mut component) = query.get_mut(event.target()) {
                    func(&mut event, &mut component);
                } else {
                    #[cfg(feature = "trace")]
                    error!(
                        "Component {:?} not found on entity {:?} during callback for event {:?}",
                        std::any::type_name::<C>(),
                        event.target(),
                        std::any::type_name::<E>()
                    );
                }
            },
        )
    }

    /// Get mutable access to the listener entity's [`EntityCommands`] using a closure any time this
    /// event listener is triggered.
    pub fn listener_commands_mut(
        &mut self,
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut EntityCommands),
    ) -> ListenerHandle<E> {
        self.run(
            move |mut event: ResMut<ListenerInput<E>>, mut commands: Commands| {
                let listener = event.listener();
                func(&mut event, &mut commands.entity(listener));
            },
        )
    }

    /// Insert a bundle on the listener entity any time this event listener is triggered.
    pub fn listener_insert(&mut self, bundle: impl Bundle + Clone) -> ListenerHandle<E> {
        self.run(
            move |event: Res<ListenerInput<E>>, mut commands: Commands| {
                let bundle = bundle.clone();
                commands.entity(event.listener()).insert(bundle);
            },
        )
    }

    /// Remove a bundle from the listener entity any time this event listener is triggered.
    pub fn listener_remove<B: Bundle>(&mut self) -> ListenerHandle<E> {
        self.run(|event: Res<ListenerInput<E>>, mut commands: Commands| {
            commands.entity(event.listener()).remove::<B>();
        })
    }

    /// Get mutable access to a specific component on the listener entity using a closure any time
    /// this event listener is triggered. If the component does not exist, an error will be logged.
    pub fn listener_component_mut<C: Component>(
        &mut self,
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut C),
    ) -> ListenerHandle<E> {
        self.run(
            move |mut event: ResMut<ListenerInput<E>>, mut query: Query<&mut C>| {
                if let Ok(mut component) = query.get_mut(event.listener()) {
                    func(&mut event, &mut component);
                } else {
                    #[cfg(feature = "trace")]
                    error!(
                        "Component {:?} not found on entity {:?} during callback for event {:?}",
                        std::any::type_name::<C>(),
                        event.listener(),
                        std::any::type_name::<E>()
                    );
                }
            },
        )
    }

    /// Send an event `F` any time this event listener is triggered.
    pub fn send_event<F: Event + From<ListenerInput<E>>>(&mut self) -> ListenerHandle<E> {
        self.run(
            move |event: Res<ListenerInput<E>>, mut ev: EventWriter<F>| {
                ev.send(F::from(event.to_owned()));
            },
        )
    }

    /// Take the boxed system callback out of this listener, leaving an empty one behind.
    pub(crate) fn take(&mut self, handle: ListenerHandle<E>) -> CallbackSystem {
        let mut temp = CallbackSystem::Empty;
        std::mem::swap(&mut self.callbacks, &mut temp);
        temp
    }

}
