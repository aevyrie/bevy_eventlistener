use bevy::{input::{keyboard::KeyboardInput, mouse::{MouseButton, MouseButtonInput}}, prelude::*};
use bevy_eventlistener::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EventListenerPlugin::<MyEvent<1>>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_events)
        .insert_resource(TargetEntity(Entity::PLACEHOLDER))
        .run();
}

#[derive(Clone, Event, EntityEvent)]
#[can_bubble]
struct MyEvent<const N: usize> {
    target: Entity,
}

#[derive(Resource)]
struct TargetEntity(Entity);

fn setup(mut commands: Commands, mut target_entity: ResMut<TargetEntity>, my_query: Query<On>) {

    // create empty listener
    let listener = commands.spawn((Name::new("Foo"), On::<MyEvent<1>>::default()));

    // Option 2 is to create a listener with callbacks ahead of time, and insert that
    // Create an empty listener
    let mut bar_listener = On::<MyEvent<1>>::default();
    // Add callbacks to it via `Commands`, and return the id of the callback you just added
    let callback_1 = commands.add_callback(bar_listener, || info!("hello from a callback 1"));
    // Repeat as many times as you'd like
    let callback_2 = commands.add_callback(bar_listener, || info!("hello from a callback 2"));
    // Add the listener, which now has two callbacks, to a new entity:
    commands.spawn((Name::new("Bar"), bar_listener));
    
    // Option 3 is to add to an existing listener which you got from a query or similar
    // Create an empty listener
    let mut baz_listener = my_query.single_mut();
    // Same API as above: we have mutable access to the listener, so we can add callbacks to it
    let callback_3 = commands.add_callback(baz_listener, || info!("hello from a callback 3"));
    
    // Note that commands.add_callback() could be provided in a few ways:
    // same as above:       Commands::add_callback(&mut self, listener: &mut On<E>) -> ListenerId
    // on the listener:     On::<E>::add_callback(&mut self, &mut World) -> ListenerId
    
    // Because `add_callback` returns the id, you can then use it to remove or replace callbacks
    let callback_id = commands.add_callback(listener, || info!("hello world"));
    commands.remove_callback(listener, callback_id); // also despawns the callback entity
    commands.replace_callback(listener, callback_id, || info!("a new callback"));

}

fn some_simple_system(time: Res<Time>) {
    info!("Hello from a bevy system! {:?} has elapsed", time.elapsed());
}

#[allow(clippy::too_many_arguments)]
fn keyboard_events(
    target: Res<TargetEntity>,
    mut inputs: EventReader<KeyboardInput>,
    mut ev1: EventWriter<MyEvent<1>>,
) {
    let target = target.0;
    for input in inputs
        .read()
        .filter(|input| !input.state.is_pressed())
        .map(|input| input.key_code)
    {ev1.send(MyEvent {
        target
    });}
}

/*
fn setup(mut commands: Commands, mut listener: ResMut<Click>, ) {
    // In the simplest case, you can add a listener with no callbacks in a bundle
    commands.spawn((Name::new("Foo"), On::<Click>::default()));
    
    // Option 2 is to create a listener with callbacks ahead of time, and insert that
    // Create an empty listener
    let mut bar_listener = On::<Click>::default();
    // Add callbacks to it via `Commands`, and return the id of the callback you just added
    let callback_1 = commands.add_callback(bar_listener, || info!("hello from a callback 1"));
    // Repeat as many times as you'd like
    let callback_2 = commands.add_callback(bar_listener, || info!("hello from a callback 2"));
    // Add the listener, which now has two callbacks, to a new entity:
    commands.spawn((Name::new("Bar"), bar_listener));
    
    // Option 3 is to add to an existing listener which you got from a query or similar
    // Create an empty listener
    let mut baz_listener = my_query.single_mut();
    // Same API as above: we have mutable access to the listener, so we can add callbacks to it
    let callback_3 = commands.add_callback(baz_listener, || info!("hello from a callback 3"));
    
    // Note that commands.add_callback() could be provided in a few ways:
    // same as above:       Commands::add_callback(&mut self, listener: &mut On<E>) -> ListenerId
    // on the listener:     On::<E>::add_callback(&mut self, &mut World) -> ListenerId
    
    // Because `add_callback` returns the id, you can then use it to remove or replace callbacks
    let callback_id = commands.add_callback(listener, || info!("hello world"));
    commands.remove_callback(listener, callback_id); // also despawns the callback entity
    commands.replace_callback(listener, callback_id, || info!("a new callback"));
} 
*/