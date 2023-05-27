use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{listener_graph::EventDispatcher, EntityEvent};

/// Bubbles [`EntityEvent`]s up the entity hierarchy, running  callbacks.
pub fn bubble_events<E: EntityEvent + 'static>(world: &mut World) {
    world.resource_scope(|world, mut dispatcher: Mut<EventDispatcher<E>>| {
        let dispatcher = dispatcher.as_mut();
        dispatcher.events.iter().for_each(|(event, root_node)| {
            let mut this_node = *root_node;

            world.insert_resource(Listened {
                listener: this_node,
                event_data: event.to_owned(),
                bubble: Bubble::Up,
            });
            while let Some((callback, next_node)) = dispatcher.listener_graph.get_mut(&this_node) {
                world.resource_mut::<Listened<E>>().listener = this_node;
                callback.run(world);
                if !event.can_bubble() || world.resource::<Listened<E>>().bubble == Bubble::Burst {
                    break;
                }
                match next_node {
                    Some(next_node) => this_node = *next_node,
                    _ => break,
                }
            }
            world.remove_resource::<Listened<E>>();
        });
    });
}

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
    pub(crate) bubble: Bubble,
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
        self.bubble.burst()
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

/// Determines whether an event should continue to bubble up the entity hierarchy.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bubble {
    /// Allows this event to bubble up to its parent.
    #[default]
    Up,
    /// Stops this event from bubbling to the next parent.
    Burst,
}

impl Bubble {
    /// Stop this event from bubbling to the next parent.
    pub fn burst(&mut self) {
        *self = Bubble::Burst;
    }
}
