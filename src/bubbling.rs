use bevy::{prelude::*, utils::HashMap};

use crate::{callbacks::CallbackSystem, EntityEvent, On};

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

/// In order to traverse the entity hierarchy and read events without requiring `Clone`, we need to
/// extract the callbacks out of their components before they can be run. This is because running
/// callbacks requires mutable access to the [`World`], which we can't do if we are also trying to
/// mutate the [`On`]'s inner callback state via `run` at the same time.
#[derive(Resource)]
pub struct ListenerGraph<E: EntityEvent> {
    /// All the events of type `E` that were emitted this frame, and encountered an [`On<E>`] while
    /// traversing the entity hierarchy. The `Entity` in the tuple is the root node to use when
    /// traversing the listener graph.
    pub(crate) events: Vec<(E, Entity)>,
    /// Traversing the entity hierarchy for each event can visit the same entity multiple times.
    /// Storing the callbacks for each of these potentially visited entities in a graph structure is
    /// necessary for a few reasons:
    ///
    /// - Callback systems cannot implement `Clone`, so we can only have one copy of each callback
    ///   system.
    /// - For complex hierarchies, this is more memory efficient.
    /// - This allows us to jump to the next listener in the hierarchy without unnecessary
    ///   traversal. When bubbling many events of the same type `E` through the same entity tree,
    ///   this can save a significant amount of work.
    pub(crate) listener_graph: HashMap<Entity, (CallbackSystem<E>, Option<Entity>)>,
}

impl<E: EntityEvent> ListenerGraph<E> {
    /// For each event, we need to build a chain of event listeners in the entity tree starting at
    /// the event's target. This does not need a node for every entity in the tree, instead, only
    /// the entities with event listeners are included.
    pub(crate) fn build(
        mut events: EventReader<E>,
        mut listeners: Query<(Option<&mut On<E>>, Option<&Parent>)>,
        mut callbacks: ResMut<ListenerGraph<E>>,
    ) {
        callbacks.events.clear();
        callbacks.listener_graph.clear();

        for event in events.iter() {
            build_branch_depth_first(event, &mut callbacks, &mut listeners);
        }
    }

    /// Once we are done bubbling, we need to add the callback systems back into the components we
    /// moved them from when building the tree.
    pub(crate) fn cleanup(
        mut listeners: Query<&mut On<E>>,
        mut callbacks: ResMut<ListenerGraph<E>>,
    ) {
        for (entity, (callback, _)) in callbacks.listener_graph.drain() {
            if let Ok(mut listener) = listeners.get_mut(entity) {
                listener.callback = callback;
            }
        }
    }
}

/// Build a branch of the event bubbling graph, starting from the target entity, traversing up the
/// hierarchy through the parents. Any event listeners that are found during traversal will be added
/// as nodes to the graph.
fn build_branch_depth_first<E: EntityEvent>(
    event: &E,
    callbacks: &mut ResMut<ListenerGraph<E>>,
    listeners: &mut Query<(Option<&mut On<E>>, Option<&Parent>)>,
) {
    let graph = &mut callbacks.listener_graph;
    let mut this_node = event.target();
    let mut prev_node = None;
    let mut first_listener = None;

    loop {
        if let Some((_, next_node)) = graph.get(&this_node) {
            // If the current entity is already in the map, use it to jump ahead
            if first_listener.is_none() {
                first_listener = Some(this_node);
            }
            match next_node {
                Some(next_node) => this_node = *next_node,
                None => break, // Bubble reached the surface!
            }
        } else if let Ok((event_listener, parent)) = listeners.get_mut(this_node) {
            // Otherwise, get the current entity's data with a query
            if let Some(mut event_listener) = event_listener {
                // If it has an event listener, we need to add it to the map
                graph.insert(this_node, (event_listener.take(), None));
                // We must also point the previous node to this node
                if let Some((_, prev_nodes_next_node @ None)) =
                    prev_node.and_then(|e| graph.get_mut(&e))
                {
                    *prev_nodes_next_node = Some(this_node);
                }
                if first_listener.is_none() {
                    first_listener = Some(this_node);
                }
                prev_node = Some(this_node);
            }
            match parent {
                Some(parent) => this_node = **parent,
                None => break, // Bubble reached the surface!
            }
        } else {
            // This can be reached if the entity targeted by the event was deleted before
            // the bubbling system could run.
            break;
        }

        if !event.bubbles() {
            break;
        }
    }

    if let Some(first_listener) = first_listener {
        // Only add events if they interact with an event listener.
        callbacks.events.push((event.to_owned(), first_listener));
    }
}

impl<E: EntityEvent> Default for ListenerGraph<E> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            listener_graph: HashMap::new(),
        }
    }
}
