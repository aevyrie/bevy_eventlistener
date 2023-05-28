use std::marker::PhantomData;

use bevy::prelude::*;

use crate::EntityEvent;

pub enum CallbackSystem<E: EntityEvent> {
    Empty(PhantomData<E>),
    New(Box<dyn System<In = (), Out = ()>>),
    Initialized(Box<dyn System<In = (), Out = ()>>),
}

impl<E: EntityEvent> CallbackSystem<E> {
    pub(crate) fn is_initialized(&self) -> bool {
        matches!(self, CallbackSystem::Initialized(_))
    }

    pub(crate) fn run(&mut self, world: &mut World) {
        if !self.is_initialized() {
            let mut temp = CallbackSystem::Empty(PhantomData);
            std::mem::swap(self, &mut temp);
            if let CallbackSystem::New(mut system) = temp {
                system.initialize(world);
                *self = CallbackSystem::Initialized(system);
            }
        }
        if let CallbackSystem::Initialized(system) = self {
            system.run((), world);
            system.apply_buffers(world);
        }
    }
}

pub type ListenedEvent<'w, E> = Res<'w, Listened<E>>;

/// Data from an event that triggered an [`On<Event>`](crate::on_event::On) listener, and is
/// currently bubbling through the entity hierarchy.
///
/// This is accessed as a bevy resource in the callback system. This resource is only available to
/// callback systems.
///
/// ```
/// # struct MyEvent {
/// #     foo: usize,
/// # }
/// fn my_callback(mut event: ResMut<Listened<MyEvent>>) {
///     event.foo += 1; // Mutate the event that is being bubbled
///     event.target(); // The entity that was originally targeted
///     event.listener(); // The entity that was listening for this event
///     event.stop_propagation(); // Stop the event from bubbling further
/// }
/// ```
#[derive(Clone, PartialEq, Debug, Resource)]
pub struct Listened<E: EntityEvent> {
    /// The entity that was listening for this event.
    pub(crate) listener: Entity,
    /// Event-specific information.
    pub(crate) event_data: E,
    pub(crate) propagate: bool,
}

impl<E: EntityEvent> Listened<E> {
    /// The entity that was listening for this event. Call `target()` to get the entity that this
    /// event originally targeted before it started bubbling through the hierarchy. Note that the
    /// target and listener can be the same entity.
    pub fn listener(&self) -> Entity {
        self.listener
    }

    /// When called, the event will stop bubbling up the hierarchy to its parent.
    pub fn stop_propagation(&mut self) {
        self.propagate = false;
    }
}

impl<E: EntityEvent> std::ops::Deref for Listened<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.event_data
    }
}

impl<E: EntityEvent> std::ops::DerefMut for Listened<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event_data
    }
}
