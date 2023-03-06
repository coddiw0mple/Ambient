use std::collections::HashSet;

use ambient_ecs::{components, world_events, Debuggable, Description, Entity, Name, Networked, Store, System, SystemGroup};
use glam::{vec2, Vec2};
use serde::{Deserialize, Serialize};
pub use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use winit::event::{ModifiersState, ScanCode};

pub mod picking;

#[derive(Debug, Clone)]
/// Represents a keyboard event with attached modifiers
pub struct KeyboardEvent {
    pub scancode: ScanCode,
    pub state: ElementState,
    pub keycode: Option<VirtualKeyCode>,
    pub modifiers: ModifiersState,
    pub is_focused: bool,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PlayerRawInput {
    pub keys: HashSet<VirtualKeyCode>,
    pub mouse_position: Vec2,
    pub mouse_wheel: f32,
    pub mouse_buttons: HashSet<MouseButton>,
}

components!("input", {
    event_received_character: char,
    event_keyboard_input: KeyboardEvent,
    @[Debuggable, Networked, Store, Name["Event mouse input"], Description["A mouse button was pressed (true) or released (false). Will also contain a `mouse_button` component."]]
    event_mouse_input: bool,
    @[Debuggable, Networked, Store, Name["Event mouse motion"], Description["The mouse was moved. The value represents the delta. Use mouse_position to get the current position."]]
    event_mouse_motion: Vec2,
    @[Debuggable, Networked, Store, Name["Event mouse wheel"], Description["The mouse wheel moved. The value represents the delta."]]
    event_mouse_wheel: Vec2,
    @[Debuggable, Networked, Store, Name["Event mouse wheel"], Description["If true, the mouse_wheel_event should be interpreted as pixels, if false it should be interpreted as lines."]]
    event_mouse_wheel_pixels: bool,
    event_modifiers_change: ModifiersState,
    event_focus_change: bool,


    @[Debuggable, Networked, Store, Name["Mouse button"], Description["The mouse button. 0=left, 1=right, 2=middle."]]
    mouse_button: u32,

    player_raw_input: PlayerRawInput,
    player_prev_raw_input: PlayerRawInput,
});

pub fn init_all_components() {
    picking::init_components();
    init_components();
}

pub fn event_systems() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new("inputs", vec![Box::new(InputSystem::new())])
}

#[derive(Debug)]
pub struct InputSystem {
    modifiers: ModifiersState,
    is_focused: bool,
}

impl InputSystem {
    pub fn new() -> Self {
        Self { modifiers: ModifiersState::empty(), is_focused: true }
    }
}

impl System<Event<'static, ()>> for InputSystem {
    fn run(&mut self, world: &mut ambient_ecs::World, event: &Event<'static, ()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                &WindowEvent::Focused(focused) => {
                    self.is_focused = focused;
                    world.resource_mut(world_events()).add_event(Entity::new().with(event_focus_change(), focused));
                }
                WindowEvent::ReceivedCharacter(c) => {
                    world.resource_mut(world_events()).add_event(Entity::new().with(event_received_character(), *c));
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    let event = KeyboardEvent {
                        scancode: input.scancode,
                        state: input.state,
                        keycode: input.virtual_keycode,
                        modifiers: self.modifiers,
                        is_focused: self.is_focused,
                    };
                    world.resource_mut(world_events()).add_event(Entity::new().with(event_keyboard_input(), event));
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    world.resource_mut(world_events()).add_event(
                        Entity::new()
                            .with(
                                event_mouse_input(),
                                match state {
                                    ElementState::Pressed => true,
                                    ElementState::Released => false,
                                },
                            )
                            .with(
                                mouse_button(),
                                match button {
                                    MouseButton::Left => 0,
                                    MouseButton::Right => 1,
                                    MouseButton::Middle => 2,
                                    MouseButton::Other(v) => *v as u32,
                                },
                            ),
                    );
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    world.resource_mut(world_events()).add_event(
                        Entity::new()
                            .with(
                                event_mouse_wheel(),
                                match *delta {
                                    MouseScrollDelta::LineDelta(x, y) => vec2(x, y),
                                    MouseScrollDelta::PixelDelta(p) => vec2(p.x as f32, p.y as f32),
                                },
                            )
                            .with(event_mouse_wheel_pixels(), matches!(delta, MouseScrollDelta::PixelDelta(..))),
                    );
                }
                WindowEvent::ModifiersChanged(mods) => {
                    world.resource_mut(world_events()).add_event(Entity::new().with(event_modifiers_change(), *mods));
                }

                _ => {}
            },

            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                world
                    .resource_mut(world_events())
                    .add_event(Entity::new().with(event_mouse_motion(), vec2(delta.0 as f32, delta.1 as f32)));
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct MouseInput {
    pub state: ElementState,
    pub button: MouseButton,
}

pub fn mouse_button_from_u32(button: u32) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        x => MouseButton::Other(x as u16),
    }
}
