//! This module provides event listeners, [`On`], the most important part of
//! [`bevy_eventlistener`](crate).

use std::marker::PhantomData;

use crate::callbacks::{CallbackSystem, ListenerInput};
use bevy_ecs::{prelude::*, system::EntityCommands, world::Command};
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
    /// A function that is called when the event listener is triggered.
    pub(crate) callback: CallbackSystem,
}

impl<E: EntityEvent> On<E> {
    /// Run a callback system every time this event listener is triggered. This can be a closure or
    /// a function, as described by bevy's documentation. The only notable difference from Bevy
    /// systems is that the callback system can access a resource with event data,
    /// [`ListenerInput`]. You can more easily access this with the system params
    /// [`Listener`](crate::callbacks::Listener) and [`ListenerMut`](crate::callbacks::ListenerMut).
    pub fn run<Marker>(callback: impl IntoSystem<(), (), Marker>) -> Self {
        Self {
            phantom: PhantomData,
            callback: CallbackSystem::New(Box::new(IntoSystem::into_system(callback))),
        }
    }

    /// Add a single [`Command`] any time this event listener is triggered. The command must
    /// implement `From<E>`.
    pub fn add_command<C: From<ListenerInput<E>> + Command + Send + Sync + 'static>() -> Self {
        Self::run(|event: Res<ListenerInput<E>>, mut commands: Commands| {
            commands.add(C::from(event.to_owned()));
        })
    }

    /// Get mutable access to [`Commands`] any time this event listener is triggered.
    pub fn commands_mut(
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut Commands),
    ) -> Self {
        Self::run(
            move |mut event: ResMut<ListenerInput<E>>, mut commands: Commands| {
                func(&mut event, &mut commands);
            },
        )
    }

    /// Get mutable access to the target entity's [`EntityCommands`] using a closure any time this
    /// event listener is triggered.
    pub fn target_commands_mut(
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut EntityCommands),
    ) -> Self {
        Self::run(
            move |mut event: ResMut<ListenerInput<E>>, mut commands: Commands| {
                let target = event.target();
                func(&mut event, &mut commands.entity(target));
            },
        )
    }

    /// Insert a bundle on the target entity any time this event listener is triggered.
    pub fn target_insert(bundle: impl Bundle + Clone) -> Self {
        Self::run(
            move |event: Res<ListenerInput<E>>, mut commands: Commands| {
                let bundle = bundle.clone();
                commands.entity(event.target()).insert(bundle);
            },
        )
    }

    /// Remove a bundle from the target entity any time this event listener is triggered.
    pub fn target_remove<B: Bundle>() -> Self {
        Self::run(|event: Res<ListenerInput<E>>, mut commands: Commands| {
            commands.entity(event.target()).remove::<B>();
        })
    }

    /// Get mutable access to a specific component on the target entity using a closure any time
    /// this event listener is triggered. If the component does not exist, an error will be logged.
    pub fn target_component_mut<C: Component>(
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut C),
    ) -> Self {
        Self::run(
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
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut EntityCommands),
    ) -> Self {
        Self::run(
            move |mut event: ResMut<ListenerInput<E>>, mut commands: Commands| {
                let listener = event.listener();
                func(&mut event, &mut commands.entity(listener));
            },
        )
    }

    /// Insert a bundle on the listener entity any time this event listener is triggered.
    pub fn listener_insert(bundle: impl Bundle + Clone) -> Self {
        Self::run(
            move |event: Res<ListenerInput<E>>, mut commands: Commands| {
                let bundle = bundle.clone();
                commands.entity(event.listener()).insert(bundle);
            },
        )
    }

    /// Remove a bundle from the listener entity any time this event listener is triggered.
    pub fn listener_remove<B: Bundle>() -> Self {
        Self::run(|event: Res<ListenerInput<E>>, mut commands: Commands| {
            commands.entity(event.listener()).remove::<B>();
        })
    }

    /// Get mutable access to a specific component on the listener entity using a closure any time
    /// this event listener is triggered. If the component does not exist, an error will be logged.
    pub fn listener_component_mut<C: Component>(
        mut func: impl 'static + Send + Sync + FnMut(&mut ListenerInput<E>, &mut C),
    ) -> Self {
        Self::run(
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

    /// Send an event `F`  any time this event listener is triggered.
    pub fn send_event<F: Event + From<ListenerInput<E>>>() -> Self {
        Self::run(
            move |event: Res<ListenerInput<E>>, mut ev: EventWriter<F>| {
                ev.send(F::from(event.to_owned()));
            },
        )
    }

    /// Take the boxed system callback out of this listener, leaving an empty one behind.
    pub(crate) fn take(&mut self) -> CallbackSystem {
        let mut temp = CallbackSystem::Empty;
        std::mem::swap(&mut self.callback, &mut temp);
        temp
    }
}
