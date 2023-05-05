use bevy::prelude::*;

use crate::{
    bubbling::{Bubble, ListenerGraph},
    EntityEvent,
};

/// Bubbles [`EntityEvent`]s up the entity hierarchy, running  callbacks.
pub fn execute<E: EntityEvent + 'static>(world: &mut World) {
    world.resource_scope(|world, mut callbacks: Mut<ListenerGraph<E>>| {
        world.insert_resource(ListenerGraph::<E>::default());

        let callbacks = &mut *callbacks;

        for (event, root_node) in callbacks.events.iter() {
            let mut this_node = *root_node;
            'bubble_traversal: while let Some((callback, next_node)) =
                callbacks.listener_graph.get_mut(&this_node)
            {
                let event = Listened {
                    listener: this_node,
                    event_data: event.to_owned(),
                };
                let callback_result = callback.run(world, event);
                if callback_result == Bubble::Burst {
                    break 'bubble_traversal;
                }
                match next_node {
                    Some(next_node) => this_node = *next_node,
                    _ => break 'bubble_traversal,
                }
            }
        }
    });
}

pub enum CallbackSystem<E: EntityEvent> {
    Empty,
    New(Box<dyn System<In = Listened<E>, Out = Bubble>>),
    Initialized(Box<dyn System<In = Listened<E>, Out = Bubble>>),
}

impl<E: EntityEvent> CallbackSystem<E> {
    pub(crate) fn is_initialized(&self) -> bool {
        matches!(self, CallbackSystem::Initialized(_))
    }

    pub(crate) fn run(&mut self, world: &mut World, event_data: Listened<E>) -> Bubble {
        if !self.is_initialized() {
            let mut temp = CallbackSystem::Empty;
            std::mem::swap(self, &mut temp);
            if let CallbackSystem::New(mut system) = temp {
                system.initialize(world);
                *self = CallbackSystem::Initialized(system);
            }
        }
        match self {
            CallbackSystem::Initialized(system) => {
                let result = system.run(event_data, world);
                system.apply_buffers(world);
                result
            }
            _ => unreachable!(),
        }
    }
}

/// Wraps an [`EntityEvent`] to include the listener. This is passed into the event listener's
/// callback when [`On<EntityEvent>`] is triggered.
#[derive(Clone, PartialEq, Debug)]
pub struct Listened<E: EntityEvent> {
    /// The entity that was listening for this event.
    pub(crate) listener: Entity,
    /// Event-specific information.
    pub(crate) event_data: E,
}

impl<E: EntityEvent> Listened<E> {
    pub fn listener(&self) -> Entity {
        self.listener
    }
}

impl<E: EntityEvent> std::ops::Deref for Listened<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.event_data
    }
}
