# Ideas

## Button + Door

A pressure plate on the track that opens a door somewhere else in the level. The finish
(or a shortcut) is behind the door, so the player has to find and press the button first.

### Gameplay

- The button is a flat plate set into the floor. Rolling onto it presses it (sinks, changes colour, plays a sound).
- Each button is linked to one or more doors through a shared `channel` number, so one level can have several pairs.
- The door is a wall that slides down into the floor (or up) when its channel is active.
- Fits the maze idea from level_06: the button sits at the end of a branch that looks like a dead end,
  so the "wrong" path turns out to be required.
- Restarting the level (R or death) resets all buttons and doors.

### Button modes

| Mode | Behaviour | Use |
|---|---|---|
| `Toggle` | Stays pressed forever | Basic "find the key" puzzle |
| `Hold` | Door only open while something sits on the button | Push a CubePile cube or domino onto it |
| `Timed { seconds }` | Door closes again after a delay | Race from the button to the door |

`Hold` makes the existing loose cubes (CubePile, Dominoes) useful as puzzle pieces, not just clutter.

### Components (Skein, set in Blender)

```rust
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Button {
    channel: u32,
    mode: ButtonMode,
}

#[derive(Reflect, Debug)]
enum ButtonMode {
    Toggle,
    Hold,
    Timed { seconds: f32 },
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct Door {
    channel: u32,
    #[reflect(default)]
    open_offset: Vec3, // how far the door moves when open, e.g. (0, -1.6, 0)
    #[reflect(default)]
    speed: f32,
}
```

### Implementation sketch

- **Button**: give the collider a `Sensor` + `CollisionEventsEnabled` on the `Trigger` layer, found
  through the collider's ancestors (same pattern as `setup_finish_collider` and the bouncer).
  - `CollisionStart` → mark the channel active. `CollisionEnd` → release it (Hold mode only).
  - Hold mode should count everything touching the sensor (player and loose cubes), not just the player.
- **State**: a `Resource` like `ActiveChannels(HashSet<u32>)`, cleared on `LevelLoaded` / `LevelRestarted`.
- **Door**: `RigidBody::Kinematic`. Each frame, move it toward `closed_pos + open_offset` when its
  channel is active, and back when it isn't. Kinematic movement pushes the player instead of
  teleporting through them.
- **Feedback**: swap the button material when pressed and maybe play a click. A small light or
  colour on the door that matches the button would make the link easier to read.

### Kit pieces to add (minigolf_kit.blend)

- `KIT_Button`: straight tile with the plate in the middle.
- `KIT_ButtonDeadEnd`: DeadEnd tile with the button at the closed end (the "hidden key" branch).
- `KIT_Door`: straight tile with a sliding door across the full width (rail to rail).
- Optional: `KIT_DoorCorner`, a corner tile with the door, for hiding the finish around a bend.

### Level ideas

- **Intro**: straight line. The button is visible right before the door, to teach the link.
- **level_06 remix**: the finish sits behind a door. The button is at the end of the
  SpinnerQuad dead end, so the branch that looked like a trap is the key.
- **Hold puzzle**: push a cube from a CubePile onto a `Hold` button to keep the door open.
- **Timed run**: the `Timed` button is on the far side of Ice/Sand, and you have to make it to the door before it closes.

### Open questions

- Should the door be visible from the button (cause and effect is clear) or hidden (puzzle)?
- Should the camera briefly pan to the door when a button is pressed?
- Can the player get stuck (Hold button with no cubes left)? R restarts the level, but maybe a cube respawn would be better.
- Should the door hurt or push the player if it closes on them, or just stop?
