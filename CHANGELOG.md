# 0.6.2

- Fixed: `On<E>` event listeners that mutate themselves inside a callback were being overwritten
  during cleanup.

# 0.6.1

- Fixed: event listeners adding extra delay when processing events.

# 0.6.0

- Changed: updated to bevy 0.12

# 0.5.1

- Changed: reduced overhead of callback `run` function
- Fixed: Does not compile if bevy_reflect's "documentation" feature is enabled after updating to
  0.5. This fixed by removing the dependency of the derive crate on the core crate.

# 0.5.0

- Removed: public dead code trait CallbackSystemTrait has been removed.
- Removed: many dependencies have been removed by relying on bevy sub-crates.

# 0.4.1

- Added: public `EventListenerSet` set label added, and all plugin systems added to the set.

# 0.4.0

- Changed: the plugin now runs in the `PreUpdate` schedule, instead of the `Update` schedule.
- Changed: all systems have been made public. This will allows users to rearrange the plugin for
  their needs, either running in another schedule, or building something entirely custom.

# 0.3.0

- Changed: relaxed bounds to support static `FnMut` closures for `On` methods instead of only `fn`
- Added: new `event_listener` example to guide users through how to use the supplied event listener
  methods.
- Fixed: prelude now exports `ListenerInput`
