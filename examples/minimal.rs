use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*, time::common_conditions::on_timer};
use rand::{seq::IteratorRandom, thread_rng, Rng};

use bevy_eventlistener::{
    callbacks::{Listener, ListenerMut},
    event_listener::{EntityEvent, On},
    EventListenerPlugin,
};
use bevy_eventlistener_derive::EntityEvent;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin::default())
        // Note that this plugin will add the `Attack` event to bevy:
        .add_plugins(EventListenerPlugin::<Attack>::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            attack_armor.run_if(on_timer(Duration::from_millis(200))),
        )
        .run();
}

/// An event used with event listeners must implement `EntityEvent` and `Clone`.
#[derive(Clone, Event, EntityEvent)]
#[can_bubble] // Enables event bubbling for this event, off by default
struct Attack {
    #[target] // Marks the field of the event that specifies the target entity
    target: Entity,
    damage: u16,
}

/// An entity that can take damage
#[derive(Component, Deref, DerefMut)]
struct HitPoints(u16);

/// For damage to reach the wearer, it must exceed the armor.
#[derive(Component, Deref)]
struct Armor(u16);

/// Set up the world
fn setup(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Goblin"),
            HitPoints(50),
            // This is the party trick of this crate. You can add behavior to entities via a normal
            // component. The API was designed to be read naturally:
            //
            // "When this entity is targeted with an Attack, run the take_damage system."
            //
            // The crate also provides a bunch of other helper methods; see the event_listeners
            // example or the `On` docs for more details.
            On::<Attack>::run(take_damage),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Helmet"),
                Armor(5),
                On::<Attack>::run(block_attack),
            ));
            parent.spawn((
                Name::new("Socks"),
                Armor(10),
                On::<Attack>::run(block_attack),
            ));
            parent.spawn((
                Name::new("Shirt"),
                Armor(15),
                On::<Attack>::run(block_attack),
            ));
        });
}

/// A normal bevy system that attacks a piece of armor on a timer.
fn attack_armor(entities: Query<Entity, With<Armor>>, mut attacks: EventWriter<Attack>) {
    let mut rng = rand::thread_rng();
    if let Some(target) = entities.iter().choose(&mut rng) {
        let damage = thread_rng().gen_range(1..20);
        attacks.send(Attack { target, damage });
        info!("⚔️  Attack for {} damage", damage);
    }
}

/// A callback placed on [`Armor`], checking if it absorbed all the [`Attack`] damage.
fn block_attack(mut attack: ListenerMut<Attack>, armor: Query<(&Armor, &Name)>) {
    let (armor, armor_name) = armor.get(attack.target).unwrap();
    let damage = attack.damage.saturating_sub(**armor);
    if damage > 0 {
        info!("🩸 {} damage passed through {}", damage, armor_name);
        attack.damage = damage;
    } else {
        info!("🛡️  {} damage blocked by {}", attack.damage, armor_name);
        attack.stop_propagation(); // Armor stopped the attack, the event stops here.
    }
}

/// A callback on the armor wearer, triggered when a piece of armor is not able to block an attack,
/// or the wearer is attacked directly.
fn take_damage(
    attack: Listener<Attack>,
    mut hp: Query<(&mut HitPoints, &Name)>,
    mut commands: Commands,
    mut app_exit: EventWriter<bevy::app::AppExit>,
) {
    let (mut hp, name) = hp.get_mut(attack.listener()).unwrap();
    **hp = hp.saturating_sub(attack.damage);

    if **hp > 0 {
        info!("{} has {:.1} HP", name, hp.0);
    } else {
        warn!("💀 {} has died a gruesome death", name);
        commands.entity(attack.listener()).despawn_recursive();
        app_exit.send(bevy::app::AppExit::Success);
    }
}
