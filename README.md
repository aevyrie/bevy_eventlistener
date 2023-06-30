# Event listeners, bubbling, and callbacks for Bevy

An implementation of event listeners and callbacks, allowing you to define behavior with components.

- Define custom events that can target entities.
- Add event listener components that run callbacks when the specified event reaches that entity.
- Define callbacks as normal bevy systems.
- Events bubble up entity hierarchies, allowing you to attach behavior to trees of entities. For
  example, you could put a single event listener on the root entity of a scene, that runs a callback
  if any child entity is clicked on. This works because events that target the child of a scene will bubble up the hierarchy until an event listener is found.

## Example

Taken from the `minimal` example, here we have a goblin wearing a few pieces of armor. An `Attack`
event can target any of these entities. If an `Attack` reaches a piece of armor, the armor will try
to absorb the attack. Any damage it is not able to absorb will bubble to the goblin wearing the armor.

```rs
commands
    .spawn((
        Name::new("Goblin"),
        HitPoints(50),
        On::<Attack>::run_callback(take_damage),
    ))
    .with_children(|parent| {
        parent.spawn((
            Name::new("Helmet"),
            Armor(5),
            On::<Attack>::run_callback(block_attack),
        ));
        parent.spawn((
            Name::new("Socks"),
            Armor(10),
            On::<Attack>::run_callback(block_attack),
        ));
    });
```

## UI

This library is intended to be upstreamed to bevy for use in making interactive UI. However, as
demonstrated above, event bubbling is applicable to any kind of event that needs to traverse an
entity hierarchy. This follows the basic principles of ECS patterns: it works on *any* entity with
the required components, not just UI.

This library was initially extracted from the `0.13` version of bevy_mod_picking, as it became obvious that
this is a generically useful feature.

# License

All code in this repository is dual-licensed under either:

- MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option. This means you can select the license you prefer.

## Your contributions
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

