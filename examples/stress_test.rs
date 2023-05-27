use bevy::{log::LogPlugin, prelude::*};

use bevy_eventlistener::{
    on_event::{EntityEvent, On},
    EventListenerPlugin,
};
use bevy_eventlistener_derive::EntityEvent;

const DENSE_LISTENERS: bool = false;
const ENTITY_DEPTH: usize = 20;
const ENTITY_WIDTH: usize = 500;
const N_EVENTS: usize = 1000;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin::default())
        .add_plugin(EventListenerPlugin::<EventFoo>::default())
        .add_event::<EventFoo>()
        .insert_resource(Target(Entity::PLACEHOLDER))
        .add_startup_system(move |mut commands: Commands, mut target: ResMut<Target>| {
            let event_listener = || On::<EventFoo>::run_callback(|| {});
            for _ in 0..ENTITY_WIDTH {
                let mut parent = commands.spawn(event_listener()).id();
                for i in 1..=ENTITY_DEPTH {
                    target.0 = if i == ENTITY_DEPTH || DENSE_LISTENERS {
                        commands.spawn(event_listener()).id()
                    } else {
                        commands.spawn_empty().id()
                    };
                    commands.entity(parent).add_child(target.0);
                    parent = target.0;
                }
            }
        })
        .add_system(send_events)
        .run();
}

#[derive(Clone, EntityEvent)]
struct EventFoo {
    #[target]
    target: Entity,
}

#[derive(Resource)]
struct Target(Entity);

fn send_events(target: Res<Target>, mut event: EventWriter<EventFoo>) {
    (0..N_EVENTS).for_each(|_| {
        event.send(EventFoo { target: target.0 });
    });
}
