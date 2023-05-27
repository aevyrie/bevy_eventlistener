use bevy::{app::AppExit, prelude::*};
use bevy_eventlistener::{
    on_event::{EntityEvent, On},
    EventListenerPlugin,
};
use bevy_eventlistener_derive::EntityEvent;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

#[derive(Clone, EntityEvent)]
struct EventFoo {
    #[target]
    target: Entity,
}

#[derive(Resource)]
struct Target(Entity);

fn send_events(
    target: Res<Target>,
    mut event: EventWriter<EventFoo>,
    mut exit: EventWriter<AppExit>,
) {
    event.send(EventFoo { target: target.0 });
    exit.send(AppExit);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_elem");
    for size in [1, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let event_listener = || On::<EventFoo>::run_callback(|| {});
                App::new()
                    .add_plugins(MinimalPlugins)
                    .add_plugin(EventListenerPlugin::<EventFoo>::default())
                    .add_event::<EventFoo>()
                    .insert_resource(Target(Entity::PLACEHOLDER))
                    .add_startup_system(
                        move |mut commands: Commands, mut target: ResMut<Target>| {
                            let mut parent = commands.spawn(event_listener()).id();
                            for _ in 0..size {
                                target.0 = commands.spawn(event_listener()).id();
                                commands.entity(parent).add_child(target.0);
                                parent = target.0;
                            }
                        },
                    )
                    .add_system(send_events)
                    .run()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
