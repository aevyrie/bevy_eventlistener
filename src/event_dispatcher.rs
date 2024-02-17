//! Provides the [`EventDispatcher`], which handles bubbling events through the entity hierarchy,
//! and triggering event listeners.

use bevy_ecs::prelude::*;
use bevy_hierarchy::Parent;
use bevy_utils::{HashMap, HashSet};

use crate::{
    callbacks::{CallbackSystem, ListenerInput},
    event_listener::On,
    EntityEvent,
};

/// Builds and executes the event listener callback graph.
///
/// To traverse the entity hierarchy and read events without requiring callbacks implement `Clone`,
/// we need to extract the callbacks out of their components before they can be run. This is because
/// running callbacks requires mutable access to the [`World`], which we can't do if we are also
/// trying to mutate the [`On`]'s inner callback state via `run` at the same time.
#[derive(Resource)]
pub struct EventDispatcher<E: EntityEvent> {
    /// All the events of type `E` that were emitted this frame, and encountered an [`On<E>`] while
    /// traversing the entity hierarchy. The `Entity` in the tuple is the leaf node to use when
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
    pub(crate) listener_graph: HashMap<Entity, (CallbackSystem, Option<Entity>)>,
}

impl<E: EntityEvent> EventDispatcher<E> {
    /// For each event, we need to build a chain of event listeners in the entity tree starting at
    /// the event's target. This does not need a node for every entity in the tree, instead, only
    /// the entities with event listeners are included.
    pub fn build(
        mut events: EventReader<E>,
        mut listeners: Query<(Option<&mut On<E>>, Option<&Parent>)>,
        mut dispatcher: ResMut<EventDispatcher<E>>,
        mut dead_branch_nodes: Local<HashSet<Entity>>,
        mut target_cache: Local<HashMap<Entity, Entity>>,
    ) {
        // Reuse allocated memory
        dispatcher.events.clear();
        dispatcher.listener_graph.clear();
        dead_branch_nodes.clear();
        target_cache.clear();

        for event in events.read() {
            // if the target belongs to a dead branch, exit early.
            if dead_branch_nodes.contains(&event.target()) {
                continue;
            }
            // if the target has already been used to traverse the graph, use the cached value.
            if let Some(first_listener) = target_cache.get(&event.target()) {
                dispatcher.events.push((event.to_owned(), *first_listener));
                continue;
            }
            build_branch_depth_first(
                event,
                &mut dispatcher,
                &mut listeners,
                &mut dead_branch_nodes,
                &mut target_cache,
            );
        }
    }

    /// Once we are done bubbling, we need to add the callback systems back into the components we
    /// moved them from when building the tree.
    pub fn cleanup(mut listeners: Query<&mut On<E>>, mut callbacks: ResMut<EventDispatcher<E>>) {
        for (entity, (callback, _)) in callbacks.listener_graph.drain() {
            if let Ok(mut listener) = listeners.get_mut(entity) {
                // Do not restore the callback if it has been replaced by the event handler.
                if listener.callback.is_empty() {
                    listener.callback = callback;
                }
            }
        }
    }

    /// Bubbles [`EntityEvent`]s up the entity hierarchy, running  callbacks.
    pub fn bubble_events(world: &mut World) {
        world.resource_scope(|world, mut dispatcher: Mut<EventDispatcher<E>>| {
            let dispatcher = dispatcher.as_mut();
            dispatcher.events.drain(..).for_each(|(event_data, leaf)| {
                let mut listener = leaf;
                let can_bubble = event_data.can_bubble();

                world.insert_resource(ListenerInput {
                    listener,
                    event_data,
                    propagate: true,
                });
                while let Some((callback, next_node)) = dispatcher.listener_graph.get_mut(&listener)
                {
                    world.resource_mut::<ListenerInput<E>>().listener = listener;
                    callback.run(world);
                    if !can_bubble || !world.resource::<ListenerInput<E>>().propagate {
                        break;
                    }
                    match next_node {
                        Some(next_node) => listener = *next_node,
                        _ => break,
                    }
                }
                world.remove_resource::<ListenerInput<E>>();
            });
        });
    }
}

/// Build a branch of the event bubbling graph, starting from the target entity, traversing up the
/// hierarchy through the parents. Any event listeners that are found during traversal will be added
/// as nodes to the graph.
fn build_branch_depth_first<E: EntityEvent>(
    event: &E,
    dispatcher: &mut ResMut<EventDispatcher<E>>,
    listeners: &mut Query<(Option<&mut On<E>>, Option<&Parent>)>,
    dead_branch_nodes: &mut HashSet<Entity>,
    target_cache: &mut HashMap<Entity, Entity>,
) {
    let graph = &mut dispatcher.listener_graph;
    let mut this_node = event.target();
    let mut prev_node = None;
    let mut first_listener = None;

    loop {
        if let Some((_, next_node)) = graph.get(&this_node) {
            // If the current entity is already in the map, use it to jump ahead
            if first_listener.is_none() {
                first_listener = Some(this_node);
            }
            if prev_node.is_none() {
                break; // We can break if we aren't in the middle of mapping a path
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
                None => {
                    if first_listener.is_none() {
                        // No listeners were found when traversing the entire branch. To prevent
                        // other events re-traversing this dead branch, we record the target as
                        // belonging to a dead branch.
                        dead_branch_nodes.insert(event.target());
                    }
                    break; // Bubble reached the surface!
                }
            }
        } else {
            // This branch can only be reached if the listeners.get_mut() call fails. Note that the
            // query allows all components to be optional, which means this can only fail if the
            // entity no longer exists. This can happen if the entity targeted by the event was
            // deleted before the bubbling system could run.
            break;
        }

        if !event.can_bubble() {
            break;
        }
    }

    if let Some(first_listener) = first_listener {
        // Only add events if they interact with an event listener.
        dispatcher.events.push((event.to_owned(), first_listener));
        target_cache.insert(event.target(), first_listener);
    }
}

impl<E: EntityEvent> Default for EventDispatcher<E> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            listener_graph: HashMap::new(),
        }
    }
}
