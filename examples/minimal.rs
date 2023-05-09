use std::time::Duration;

use bevy::{app::AppExit, log::LogPlugin, prelude::*, time::common_conditions::on_timer};
use rand::{seq::IteratorRandom, thread_rng, Rng};

use bevy_eventlistener::{
    bubbling::Bubble, callbacks::Listened, EntityEvent, EventListenerPlugin, On,
};
use bevy_eventlistener_derive::EntityEvent;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(LogPlugin::default())
        .add_plugin(EventListenerPlugin::<Attack>::default())
        .add_event::<Attack>()
        .add_startup_system(setup)
        .add_system(attack_armor.run_if(on_timer(Duration::from_secs(1))))
        .run();
}

/// An event used with event listeners must implement `EntityEvent` and `Clone`.
#[derive(Clone, EntityEvent)]
struct Attack {
    #[target]
    target: Entity,
    power: f32,
    damage: f32,
}

/// An entity that can take damage
#[derive(Component, Deref)]
struct HitPoints(f32);

/// Armor the entity is wearing. For damage to reach the entity wearing this armor, the damage must
/// be greater than the armor.
#[derive(Component, Deref)]
struct ArmorClass(f32);

/// Attack a piece of armor.
fn attack_armor(entities: Query<Entity, With<ArmorClass>>, mut attack: EventWriter<Attack>) {
    let mut rng = rand::thread_rng();
    if let Some(entity) = entities.iter().choose(&mut rng) {
        attack.send(Attack {
            target: entity,
            power: thread_rng().gen_range(1.0..20.0),
            damage: thread_rng().gen_range(1.0..20.0),
        });
    }
}

/// Set up the world
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Bob"),
            HitPoints(50.0),
            On::<Attack>::run_callback(take_damage),
        ))
        .with_children(|builder| {
            builder.spawn((
                Name::new("Hat"),
                ArmorClass(5.0),
                On::<Attack>::run_callback(block_attack),
            ));
            builder.spawn((
                Name::new("Pants"),
                ArmorClass(10.0),
                On::<Attack>::run_callback(block_attack),
            ));
            builder.spawn((
                Name::new("Shirt"),
                ArmorClass(15.0),
                On::<Attack>::run_callback(block_attack),
            ));
        });
}

/// A callback runs on [`Attack`]s, checking if the armor stopped the attack hurting the wearer.
fn block_attack(In(attack): In<Listened<Attack>>, armor: Query<(&ArmorClass, &Name)>) -> Bubble {
    let (armor_class, armor_name) = armor.get(attack.target).unwrap();
    if attack.power > **armor_class {
        info!(
            "Attack with power {:.1} made it past the {} armor with an AC of {}",
            attack.power, armor_name, **armor_class
        );
        Bubble::Up // The attack made it through the armor! The event will bubble up to the wearer.
    } else {
        info!("{} blocked an attack.", armor_name);
        Bubble::Burst // Armor stopped the attack, the event stops here.
    }
}

/// A callback on the armor parent, triggered when a piece of armor is not able to block an attack.
fn take_damage(
    In(attack): In<Listened<Attack>>,
    mut hp: Query<(&mut HitPoints, &Name)>,
    mut exit: EventWriter<AppExit>,
) -> Bubble {
    let (mut hp, name) = hp.get_mut(attack.listener()).unwrap();
    if hp.0 > attack.damage {
        hp.0 -= attack.damage;
        info!("-> Ouch! {} has {:.1} HP.", name, hp.0);
    } else {
        warn!("{} has died a gruesome death.", name);
        exit.send(AppExit);
    }
    Bubble::Burst
}
