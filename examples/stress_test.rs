use bevy::{log::LogPlugin, prelude::*};

use bevy_eventlistener::{
    event_listener::{EntityEvent, On},
    EventListenerPlugin,
};
use bevy_eventlistener_derive::EntityEvent;
use rand::{seq::IteratorRandom, Rng};

const LISTENER_DENSITY: f64 = 0.20; // percent of nodes with listeners
const ENTITY_DEPTH: usize = 64;
const ENTITY_WIDTH: usize = 200;
const N_EVENTS: usize = 500;

#[derive(Clone, EntityEvent)]
struct TestEvent<const N: usize> {
    #[target]
    target: Entity,
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin::default())
        .add_plugin(StressTestPlugin::<1>)
        .add_plugin(StressTestPlugin::<2>)
        .add_plugin(StressTestPlugin::<3>)
        .add_plugin(StressTestPlugin::<4>)
        // add an event that has no listeners in the hierarchy
        .add_plugin(EventListenerPlugin::<TestEvent<9>>::default())
        .add_event::<TestEvent<9>>()
        .add_system(send_events::<9>)
        .run();
}

struct StressTestPlugin<const N: usize>;
impl<const N: usize> Plugin for StressTestPlugin<N> {
    fn build(&self, app: &mut App) {
        app.add_plugin(EventListenerPlugin::<TestEvent<N>>::default())
            .add_event::<TestEvent<N>>()
            .add_startup_system(setup::<N>)
            .add_system(send_events::<N>);
    }
}

fn send_events<const N: usize>(
    mut event: EventWriter<TestEvent<N>>,
    entities: Query<Entity, Without<Children>>,
) {
    let target = entities.iter().choose(&mut rand::thread_rng()).unwrap();
    (0..N_EVENTS).for_each(|_| {
        event.send(TestEvent::<N> { target });
    });
}

fn setup<const N: usize>(mut commands: Commands) {
    let event_listener = || On::<TestEvent<N>>::run_callback(|| {});
    for _ in 0..ENTITY_WIDTH {
        let mut parent = commands.spawn(event_listener()).id();
        for i in 1..=ENTITY_DEPTH {
            let child = if i == ENTITY_DEPTH || rand::thread_rng().gen_bool(LISTENER_DENSITY) {
                commands.spawn(event_listener()).id()
            } else {
                commands.spawn_empty().id()
            };
            commands.entity(parent).add_child(child);
            parent = child;
        }
    }
}
