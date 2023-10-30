use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_eventlistener::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{seq::IteratorRandom, Rng};

const LISTENER_DENSITY: f64 = 0.20; // percent of nodes with listeners
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

        b.iter(|| {
            black_box(app.update());
        });
    });

    group.bench_function("Single Event Type", |b| {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, spawn_listener_hierarchy)
            .add_plugins(EventListenerPlugin::<TestEvent<1>>::default())
            .add_systems(Update, send_events::<1, N_EVENTS>);

        b.iter(|| {
            black_box(app.update());
        });
    });

    group.bench_function("Single Event Type No Listeners", |b| {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, spawn_listener_hierarchy)
            .add_plugins(EventListenerPlugin::<TestEvent<9>>::default())
            .add_systems(Update, send_events::<9, N_EVENTS>);

        b.iter(|| {
            black_box(app.update());
        });
    });

    group.bench_function("Four Event Types", |b| {
        let mut app = App::new();
        const N_EVENTS_4: usize = N_EVENTS / 4;
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, spawn_listener_hierarchy)
            .add_plugins(EventListenerPlugin::<TestEvent<1>>::default())
            .add_plugins(EventListenerPlugin::<TestEvent<2>>::default())
            .add_plugins(EventListenerPlugin::<TestEvent<3>>::default())
            .add_plugins(EventListenerPlugin::<TestEvent<4>>::default())
            .add_systems(Update, send_events::<1, N_EVENTS_4>)
            .add_systems(Update, send_events::<2, N_EVENTS_4>)
            .add_systems(Update, send_events::<3, N_EVENTS_4>)
            .add_systems(Update, send_events::<4, N_EVENTS_4>);

        b.iter(|| {
            black_box(app.update());
        });
    });
}

#[derive(Clone, Event, EntityEvent)]
struct TestEvent<const N: usize> {
    #[target]
    target: Entity,
}

fn send_events<const N: usize, const N_EVENTS: usize>(
    mut event: EventWriter<TestEvent<N>>,
    entities: Query<Entity, Without<Children>>,
) {
    if let Some(target) = entities.iter().choose(&mut rand::thread_rng()) {
        (0..N_EVENTS).for_each(|_| {
            event.send(TestEvent::<N> { target });
        });
    }
}

fn empty_listener<const N: usize>() -> On<TestEvent<N>> {
    On::<TestEvent<N>>::run(|| {})
}

fn spawn_listener_hierarchy(mut commands: Commands) {
    for _ in 0..ENTITY_WIDTH {
        let mut parent = commands
            .spawn((
                empty_listener::<1>(),
                empty_listener::<2>(),
                empty_listener::<3>(),
                empty_listener::<4>(),
            ))
            .id();
        for i in 1..=ENTITY_DEPTH {
            let mut child = commands.spawn_empty();
            maybe_insert_listener::<1>(i, &mut child);
            maybe_insert_listener::<2>(i, &mut child);
            maybe_insert_listener::<3>(i, &mut child);
            maybe_insert_listener::<4>(i, &mut child);
            let child = child.id();
            commands.entity(parent).add_child(child);
            parent = child;
        }
    }
}

fn maybe_insert_listener<const N: usize>(i: usize, commands: &mut EntityCommands) {
    if i == ENTITY_DEPTH || rand::thread_rng().gen_bool(LISTENER_DENSITY) {
        commands.insert(empty_listener::<N>());
    }
}
