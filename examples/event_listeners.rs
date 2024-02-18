//! This example demonstrates how to use the many available event listeners. Press the number keys
//! to trigger events and output logs to the console.

use bevy::{input::keyboard::KeyboardInput, prelude::*};
use bevy_eventlistener::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EventListenerPlugin::<MyEvent<1>>::default())
        .add_plugins(EventListenerPlugin::<MyEvent<2>>::default())
        .add_plugins(EventListenerPlugin::<MyEvent<3>>::default())
        .add_plugins(EventListenerPlugin::<MyEvent<4>>::default())
        .add_plugins(EventListenerPlugin::<MyEvent<5>>::default())
        .add_plugins(EventListenerPlugin::<MyEvent<6>>::default())
        .add_plugins(EventListenerPlugin::<MyEvent<7>>::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                keyboard_events,
                some_complex_system.run_if(on_event::<DoSomethingComplex>()),
            ),
        )
        .add_event::<DoSomethingComplex>()
        .insert_resource(TargetEntity(Entity::PLACEHOLDER))
        .run();
}

/// A simple system we will call directly in a callback, blocking the schedule from running.
fn some_simple_system(time: Res<Time>) {
    info!("Hello from a bevy system! {:?} has elapsed", time.elapsed());
}

/// An event used with event listeners must implement `EntityEvent` and `Clone`.
#[derive(Clone, Event, EntityEvent)]
#[can_bubble]
struct MyEvent<const N: usize> {
    #[target] // Marks the field of the event that specifies the target entity
    target: Entity,
    message: String,
}

#[derive(Resource)]
struct TargetEntity(Entity);

fn setup(mut commands: Commands, mut target_entity: ResMut<TargetEntity>) {
    // Used to demonstrate that event listener closures can capture data
    let captured_data = 5000;

    target_entity.0 = commands
        .spawn((
            // The `run` method is the building block for all the other methods shown below. You can
            // use it to do anything the helper methods do, or make your own helper methods.
            //
            // *IMPORTANT NOTE*
            //
            // Callbacks are run with exclusive world access, and will block all other systems from
            // running! Callbacks should either be very simple, or better yet, prefer to use the
            // `send_event` helper to run a bevy system that is in your schedule, and thus does not
            // block execution.
            On::<MyEvent<1>>::run(some_simple_system),
            // Just like bevy systems, callbacks can be closures! Recall that the parameters can be
            // any bevy system parameters. The only difference is that callbacks can access a
            // special `ListenerInput` resource, which can be accessed with less noisy type aliases
            // `Listener` and `ListenerMut`.
            On::<MyEvent<2>>::run(
                move |mut event: ListenerMut<MyEvent<2>>, names: Query<&Name>| {
                    info!(
                        "I am a closure system that can capture variables like this one: {}",
                        captured_data
                    );
                    info!(
                        "I can also use queries and resources because I am a system. My name is: {:?}",
                        names.get(event.target)
                    );
                    // If you are working in a hierarchy and want to stop this event from bubbling
                    // any further, you can call this method:
                    event.stop_propagation();
                },
            ),
            // We can use helper methods to make callbacks even simpler. This helper will create an
            // eventlistener with a callback that inserts a `Name` component:
            On::<MyEvent<3>>::target_insert(Name::new("Katie")),
            // And this one will remove it:
            On::<MyEvent<4>>::target_remove::<Name>(),
            // We can use this helper to define a system that updates the `Name` component:
            On::<MyEvent<5>>::target_component_mut::<Name>(|event, name| {
                *name = Name::new(event.message.clone()); // We can get the data inside our event!
            }),
            // Here we can access the `EntityCommand`s on the target directly:
            On::<MyEvent<6>>::target_commands_mut(|_event, target_commands| {
                target_commands.log_components();
            }),
            // Unlike the `run` method or any of the helpers above, this will not prevent systems
            // from parallelizing, as the systems that react to this event can be scheduled
            // normally. In fact, you can get the best of both worlds by using run criteria on the
            // systems that react to your custom event. This allows you to run bevy systems in
            // response to events targeting a specific entity, while still allowing full system
            // parallelism.
            On::<MyEvent<7>>::send_event::<DoSomethingComplex>(),
            //
            // One last note. All of the above helpers that start with `target_` have an equivalent
            // `listener_` version. What does that mean? With event listeners there are always two
            // entities involved: the entity being *target*ed by the event, and the entity
            // *listen*ing for this event. When an entity is targeted, the event dispatcher bubbles
            // the event up the hierarchy to its parents. This allows parents to listen to events
            // targeting to children, and is why the listener may be different from the target.
        ))
        .id();
}

// The following section outlines what is needed to trigger a scheduled bevy system from an event
// listener. This is the recommended way of running logic with callbacks.

/// An event used to trigger our complex system.
#[derive(Event)]
struct DoSomethingComplex {
    important_data: usize,
}

// The send_event helper requires that we define how our event is constructed. This lets us pass
// data from the listened event into the event we are triggering.
impl From<ListenerInput<MyEvent<7>>> for DoSomethingComplex {
    fn from(value: ListenerInput<MyEvent<7>>) -> Self {
        // We have access to all sorts of info about the event that triggered the callback,
        // including the event itself!
        let _target = value.target();
        let _listener = value.listener();
        let inner_event_data = &value.message;
        DoSomethingComplex {
            important_data: inner_event_data.len(),
        }
    }
}

/// A complex system that we will schedule like a normal bevy system, and only run when triggered.
/// Unlike the [`some_simple_system`], this one can run in parallel with other systems because it is
/// scheduled.
fn some_complex_system(mut events: EventReader<DoSomethingComplex>) {
    for event in events.read() {
        info!("Doing complex things with data: {}", event.important_data)
    }
}

// This section needed for the example, but not important for learning

/// Trigger events with your keyboard
#[allow(clippy::too_many_arguments)]
fn keyboard_events(
    target: Res<TargetEntity>,
    mut inputs: EventReader<KeyboardInput>,
    mut ev1: EventWriter<MyEvent<1>>,
    mut ev2: EventWriter<MyEvent<2>>,
    mut ev3: EventWriter<MyEvent<3>>,
    mut ev4: EventWriter<MyEvent<4>>,
    mut ev5: EventWriter<MyEvent<5>>,
    mut ev6: EventWriter<MyEvent<6>>,
    mut ev7: EventWriter<MyEvent<7>>,
) {
    let target = target.0;
    for input in inputs
        .read()
        .filter(|input| !input.state.is_pressed())
        .map(|input| input.key_code)
    {
        match input {
            KeyCode::Digit1 => {
                ev1.send(MyEvent {
                    target,
                    message: "Key 1".into(),
                });
            }
            KeyCode::Digit2 => {
                ev2.send(MyEvent {
                    target,
                    message: "Key 2".into(),
                });
            }
            KeyCode::Digit3 => {
                ev3.send(MyEvent {
                    target,
                    message: "Key 3".into(),
                });
            }
            KeyCode::Digit4 => {
                ev4.send(MyEvent {
                    target,
                    message: "Key 4".into(),
                });
            }
            KeyCode::Digit5 => {
                ev5.send(MyEvent {
                    target,
                    message: "Key 5".into(),
                });
            }
            KeyCode::Digit6 => {
                ev6.send(MyEvent {
                    target,
                    message: "Key 6".into(),
                });
            }
            KeyCode::Digit7 => {
                ev7.send(MyEvent {
                    target,
                    message: "Key 7".into(),
                });
            }
            _ => (),
        }
    }
}
