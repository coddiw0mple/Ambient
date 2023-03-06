use std::{io::Write, sync::Arc};

use ambient_core::{
    player::{get_player_by_user_id, player},
    runtime,
    window::{cursor_position, window_logical_size, window_physical_size},
};
use ambient_ecs::{query, query_mut, Entity, SystemGroup, WorldDiff};
use ambient_element::{element_component, Element, Hooks};
use ambient_input::{
    event_focus_change, event_keyboard_input, event_mouse_input, event_mouse_motion, event_mouse_wheel, event_mouse_wheel_pixels,
    mouse_button, mouse_button_from_u32, player_prev_raw_input, player_raw_input, ElementState, PlayerRawInput,
};
use ambient_network::{client::game_client, log_network_result, rpc::rpc_world_diff, DatagramHandlers};
use ambient_std::unwrap_log_err;

use byteorder::{BigEndian, WriteBytesExt};

const PLAYER_INPUT_DATAGRAM_ID: u32 = 5;

pub fn register_datagram_handler(handlers: &mut DatagramHandlers) {
    handlers.insert(
        PLAYER_INPUT_DATAGRAM_ID,
        Arc::new(|state, _assets, user_id, data| {
            let input: PlayerRawInput = unwrap_log_err!(bincode::deserialize(&data));
            let mut state = state.lock();
            if let Some(world) = state.get_player_world_mut(user_id) {
                if let Some(player_id) = get_player_by_user_id(world, user_id) {
                    world.add_component(player_id, cursor_position(), input.cursor_position).unwrap();
                    world.set(player_id, player_raw_input(), input).ok();
                }
            }
        }),
    );
}

pub fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "player/server_systems",
        vec![query(player()).spawned().to_system(|q, world, qs, _| {
            let player_ids = q.collect_ids(world, qs);
            for player_id in player_ids {
                world.add_components(player_id, Entity::new().with_default(player_raw_input()).with_default(player_prev_raw_input())).ok();
            }
        })],
    )
}

pub fn server_systems_final() -> SystemGroup {
    SystemGroup::new(
        "player/server_systems_final",
        vec![query_mut(player_prev_raw_input(), player_raw_input()).to_system(|q, world, qs, _| {
            for (_, prev, input) in q.iter(world, qs) {
                *prev = input.clone();
            }
        })],
    )
}

#[element_component]
pub fn PlayerDataUpload(hooks: &mut Hooks) -> Element {
    hooks.use_frame(move |world| {
        if let Some(Some(gc)) = world.resource_opt(game_client()).cloned() {
            let state = gc.game_state.lock();
            if let Some(player_id) = get_player_by_user_id(&state.world, &gc.user_id) {
                let physical_size = *world.resource(window_physical_size());
                let logical_size = *world.resource(window_logical_size());
                let mut diff = WorldDiff::new();
                if state.world.get(player_id, window_physical_size()) != Ok(physical_size) {
                    diff = diff.add_component(player_id, window_physical_size(), physical_size);
                }
                if state.world.get(player_id, window_logical_size()) != Ok(logical_size) {
                    diff = diff.add_component(player_id, window_logical_size(), logical_size);
                }
                if !diff.is_empty() {
                    drop(state);
                    world.resource(runtime()).spawn(async move {
                        log_network_result!(gc.rpc(rpc_world_diff, diff).await);
                    });
                }
            }
        }
    });
    Element::new()
}

#[element_component]
pub fn PlayerRawInputHandler(hooks: &mut Hooks) -> Element {
    const PIXELS_PER_LINE: f32 = 5.0;

    let input = hooks.use_ref_with(|_| PlayerRawInput::default());
    let (has_focus, set_has_focus) = hooks.use_state(false);

    hooks.use_world_event({
        let input = input.clone();
        move |_world, event| {
            if let Some(event) = event.get_ref(event_keyboard_input()) {
                if let Some(keycode) = event.keycode {
                    let mut lock = input.lock();
                    match event.state {
                        ElementState::Pressed => {
                            lock.keys.insert(keycode);
                        }
                        ElementState::Released => {
                            lock.keys.remove(&keycode);
                        }
                    }
                }
            } else if let Some(pressed) = event.get(event_mouse_input()) {
                let mut lock = input.lock();
                if pressed {
                    lock.mouse_buttons.insert(mouse_button_from_u32(event.get(mouse_button()).unwrap()));
                } else {
                    lock.mouse_buttons.remove(&mouse_button_from_u32(event.get(mouse_button()).unwrap()));
                }
            } else if let Some(delta) = event.get(event_mouse_motion()) {
                input.lock().mouse_position += delta;
            } else if let Some(delta) = event.get(event_mouse_wheel()) {
                input.lock().mouse_wheel += match event.get(event_mouse_wheel_pixels()).unwrap() {
                    false => delta.y * PIXELS_PER_LINE,
                    true => delta.y,
                };
            } else if let Some(focus) = event.get(event_focus_change()) {
                set_has_focus(focus);
            }
        }
    });
    hooks.use_frame(move |world| {
        if !has_focus {
            return;
        }

        if let Some(Some(gc)) = world.resource_opt(game_client()).cloned() {
            let runtime = world.resource(runtime()).clone();
            let input = input.clone();
            let cursor_position = *world.resource(cursor_position());

            runtime.spawn(async move {
                let mut data = Vec::new();
                data.write_u32::<BigEndian>(PLAYER_INPUT_DATAGRAM_ID).unwrap();

                let msg = {
                    let mut input = input.lock();
                    input.cursor_position = cursor_position;
                    bincode::serialize(&*input).unwrap()
                };
                data.write_all(&msg).unwrap();
                gc.connection.send_datagram(data.into()).ok();
            });
        }
    });

    Element::new()
}
