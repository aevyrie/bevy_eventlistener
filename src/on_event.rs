use std::marker::PhantomData;

use crate::callbacks::{CallbackSystem, Listened};
use bevy::{
    ecs::system::{Command, EntityCommands},
    prelude::*,
};

/// An event that targets a specific entity, and should support event listeners and bubbling.
pub trait EntityEvent: Event + Clone {
    fn target(&self) -> Entity;
    /// Should events bubble up the entity hierarchy, starting from the target?
    fn can_bubble(&self) -> bool {
        true
    }
}

/// An event listener with a callback that is triggered when an [`EntityEvent`] bubbles past or
/// targets this entity.
#[derive(Component, Reflect)]
pub struct On<E: EntityEvent> {
    #[reflect(ignore)]
    /// A function that is called when the event listener is triggered.
    pub(crate) callback: CallbackSystem<E>,
}

impl<E: EntityEvent> On<E> {
    /// Run a callback system any time this event listener is triggered.
    pub fn run_callback<Marker>(callback: impl IntoSystem<(), (), Marker>) -> Self {
        Self {
            callback: CallbackSystem::New(Box::new(IntoSystem::into_system(callback))),
        }
    }

    /// Add a single [`Command`] any time this event listener is triggered. The command must
    /// implement `From<E>`.
    pub fn add_command<C: From<Listened<E>> + Command + Send + Sync + 'static>() -> Self {
        Self::run_callback(|event: Res<Listened<E>>, mut commands: Commands| {
            commands.add(C::from(event.to_owned()));
        })
    }

    /// Get mutable access to [`Commands`] any time this event listener is triggered.
    pub fn commands_mut(func: fn(&E, &mut Commands)) -> Self {
        Self::run_callback(move |event: Res<Listened<E>>, mut commands: Commands| {
            func(&event, &mut commands);
        })
    }

    /// Get mutable access to the target entity's [`EntityCommands`] using a closure any time this
    /// event listener is triggered.
    pub fn target_commands_mut(func: fn(&E, &mut EntityCommands)) -> Self {
        Self::run_callback(move |event: Res<Listened<E>>, mut commands: Commands| {
            func(&event, &mut commands.entity(event.target()));
        })
    }

    /// Insert a bundle on the target entity any time this event listener is triggered.
    pub fn target_insert(bundle: impl Bundle + Clone) -> Self {
        Self::run_callback(move |event: Res<Listened<E>>, mut commands: Commands| {
            let bundle = bundle.clone();
            commands.entity(event.target()).insert(bundle);
        })
    }

    /// Remove a bundle from the target entity any time this event listener is triggered.
    pub fn target_remove<B: Bundle>() -> Self {
        Self::run_callback(|event: Res<Listened<E>>, mut commands: Commands| {
            commands.entity(event.target()).remove::<B>();
        })
    }

    /// Get mutable access to a specific component on the target entity using a closure any time
    /// this event listener is triggered. If the component does not exist, an error will be logged.
    pub fn target_component_mut<C: Component>(func: fn(&E, &mut C)) -> Self {
        Self::run_callback(move |event: Res<Listened<E>>, mut query: Query<&mut C>| {
            if let Ok(mut component) = query.get_mut(event.target()) {
                func(&event, &mut component);
            } else {
                error!("Component {:?} not found on entity {:?} during pointer callback for event {:?}", std::any::type_name::<C>(), event.target(), std::any::type_name::<E>());
            }
        })
    }

    /// Send an event `F`  any time this event listener is triggered.
    pub fn send_event<F: Event + From<Listened<E>>>() -> Self {
        Self::run_callback(move |event: Res<Listened<E>>, mut ev: EventWriter<F>| {
            ev.send(F::from(event.to_owned()));
        })
    }

    /// Take the boxed system callback out of this listener, leaving an empty one behind.
    pub(crate) fn take(&mut self) -> CallbackSystem<E> {
        let mut temp = CallbackSystem::Empty(PhantomData);
        std::mem::swap(&mut self.callback, &mut temp);
        temp
    }
}
