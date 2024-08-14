#![allow(clippy::type_complexity)]

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_eventlistener::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{seq::IteratorRandom, Rng};

const DENSITY: usize = 20; // percent of nodes with listeners
const ENTITY_DEPTH: usize = 64;
const ENTITY_WIDTH: usize = 200;
const N_EVENTS: usize = 500;

criterion_group!(benches, event_listeners,);
criterion_main!(benches);

#[allow(clippy::unit_arg)]
fn event_listeners(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_listeners");
    group.warm_up_time(std::time::Duration::from_millis(500));

    group.bench_function("Baseline", |b| {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, spawn_listener_hierarchy);
        app.update();

        b.iter(|| {
            black_box(app.update());
        });
    });

    group.bench_function("Single Event Type", |b| {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(
                Startup,
                (
                    spawn_listener_hierarchy,
                    add_listeners_to_hierarchy::<DENSITY, 1>,
                )
                    .chain(),
            )
            .add_plugins(EventListenerPlugin::<TestEvent<1>>::default())
            .add_systems(First, send_events::<1, N_EVENTS>);
        app.update();

        b.iter(|| {
            black_box(app.update());
        });
    });

    group.bench_function("Single Event Type No Listeners", |b| {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(
                Startup,
                (
                    spawn_listener_hierarchy,
                    add_listeners_to_hierarchy::<DENSITY, 1>,
                ),
            )
            .add_plugins(EventListenerPlugin::<TestEvent<9>>::default())
            .add_systems(First, send_events::<9, N_EVENTS>);
        app.update();

        b.iter(|| {
            black_box(app.update());
        });
    });

    group.bench_function("Four Event Types", |b| {
        let mut app = App::new();
        const FRAC_N_EVENTS_4: usize = N_EVENTS / 4;
        const FRAC_DENSITY_4: usize = DENSITY / 4;

        app.add_plugins(MinimalPlugins)
            .add_systems(
                Startup,
                (
                    spawn_listener_hierarchy,
                    add_listeners_to_hierarchy::<FRAC_DENSITY_4, 1>,
                    add_listeners_to_hierarchy::<FRAC_DENSITY_4, 2>,
                    add_listeners_to_hierarchy::<FRAC_DENSITY_4, 3>,
                    add_listeners_to_hierarchy::<FRAC_DENSITY_4, 4>,
                ),
            )
            .add_plugins(EventListenerPlugin::<TestEvent<1>>::default())
            .add_plugins(EventListenerPlugin::<TestEvent<2>>::default())
            .add_plugins(EventListenerPlugin::<TestEvent<3>>::default())
            .add_plugins(EventListenerPlugin::<TestEvent<4>>::default())
            .add_systems(First, send_events::<1, FRAC_N_EVENTS_4>)
            .add_systems(First, send_events::<2, FRAC_N_EVENTS_4>)
            .add_systems(First, send_events::<3, FRAC_N_EVENTS_4>)
            .add_systems(First, send_events::<4, FRAC_N_EVENTS_4>);
        app.update();

        b.iter(|| {
            black_box(app.update());
        });
    });
}

#[derive(Clone, Event, EntityEvent)]
#[can_bubble]
struct TestEvent<const N: usize> {
    #[target]
    target: Entity,
}

fn send_events<const N: usize, const N_EVENTS: usize>(
    mut event: EventWriter<TestEvent<N>>,
    entities: Query<Entity, Without<Children>>,
) {
    let target = entities.iter().choose(&mut rand::thread_rng()).unwrap();
    (0..N_EVENTS).for_each(|_| {
        event.send(TestEvent::<N> { target });
    });
}

fn spawn_listener_hierarchy(mut commands: Commands) {
    for _ in 0..ENTITY_WIDTH {
        let mut parent = commands.spawn_empty().id();
        for _ in 0..ENTITY_DEPTH {
            let child = commands.spawn_empty().id();
            commands.entity(parent).add_child(child);
            parent = child;
        }
    }
}

fn empty_listener<const N: usize>() -> On<TestEvent<N>> {
    On::<TestEvent<N>>::run(|| {})
}

fn add_listeners_to_hierarchy<const DENSITY: usize, const N: usize>(
    mut commands: Commands,
    roots_and_leaves: Query<Entity, Or<(Without<Parent>, Without<Children>)>>,
    nodes: Query<Entity, (With<Parent>, With<Children>)>,
) {
    for entity in &roots_and_leaves {
        commands.entity(entity).insert(empty_listener::<N>());
    }
    for entity in &nodes {
        maybe_insert_listener::<DENSITY, N>(&mut commands.entity(entity));
    }
}

fn maybe_insert_listener<const DENSITY: usize, const N: usize>(commands: &mut EntityCommands) {
    if rand::thread_rng().gen_bool(DENSITY as f64 / 100.0) {
        commands.insert(empty_listener::<N>());
    }
}
