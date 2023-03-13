use ambient_element::{Element, ElementComponent, Hooks};
use ambient_guest_bridge::{
    components::input::{
        event_mouse_input, event_mouse_wheel, event_mouse_wheel_pixels, mouse_button, mouse_over, mouse_pickable_max, mouse_pickable_min,
    },
    ecs::{EntityId, World},
};
use ambient_window_types::MouseButton;
use cb::{cb, Cb};
use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseInput {
    Pressed,
    Released,
}
impl From<bool> for MouseInput {
    fn from(value: bool) -> Self {
        if value {
            Self::Pressed
        } else {
            Self::Released
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClickArea {
    pub inner: Element,
    pub on_mouse_enter: Vec<Cb<dyn Fn(&mut World, EntityId) + Sync + Send>>,
    pub on_mouse_leave: Vec<Cb<dyn Fn(&mut World, EntityId) + Sync + Send>>,
    pub on_mouse_hover: Vec<Cb<dyn Fn(&mut World, EntityId) + Sync + Send>>,
    pub on_mouse_input: Vec<Cb<dyn Fn(&mut World, EntityId, MouseInput, MouseButton) + Sync + Send>>,
    pub on_mouse_wheel: Vec<Cb<dyn Fn(&mut World, EntityId, Vec2, bool) + Sync + Send>>,
}
impl ClickArea {
    pub fn new(inner: Element) -> Self {
        Self {
            inner,
            on_mouse_enter: Vec::new(),
            on_mouse_leave: Vec::new(),
            on_mouse_hover: Vec::new(),
            on_mouse_input: Vec::new(),
            on_mouse_wheel: Vec::new(),
        }
    }
    pub fn on_mouse_hover<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_hover.push(cb(handle));
        self
    }
    pub fn on_mouse_enter<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_enter.push(cb(handle));
        self
    }
    pub fn on_mouse_leave<F: Fn(&mut World, EntityId) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_leave.push(cb(handle));
        self
    }
    pub fn on_mouse_input<F: Fn(&mut World, EntityId, MouseInput, MouseButton) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_input.push(cb(handle));
        self
    }
    pub fn on_mouse_wheel<F: Fn(&mut World, EntityId, Vec2, bool) + Sync + Send + 'static>(mut self, handle: F) -> Self {
        self.on_mouse_wheel.push(cb(handle));
        self
    }

    pub fn on_mouse_down<F: Fn(&mut World, EntityId, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.on_mouse_input(move |world, id, state, button| {
            if state == MouseInput::Pressed {
                handle(world, id, button)
            }
        })
    }
    pub fn on_mouse_up<F: Fn(&mut World, EntityId, MouseButton) + Sync + Send + 'static>(self, handle: F) -> Self {
        self.on_mouse_input(move |world, id, state, button| {
            if state == MouseInput::Released {
                handle(world, id, button)
            }
        })
    }
}
impl ElementComponent for ClickArea {
    fn render(self: Box<Self>, hooks: &mut Hooks) -> Element {
        let Self { inner, on_mouse_enter, on_mouse_leave, on_mouse_hover, on_mouse_input, on_mouse_wheel } = *self;
        let id = hooks.use_ref_with(|_| None);
        let mouse_over_count = hooks.use_ref_with(|_| 0);
        hooks.use_frame({
            let id = id.clone();
            let mouse_over_count = mouse_over_count.clone();
            move |world| {
                if let Some(id) = *id.lock() {
                    let next = world.get(id, mouse_over()).unwrap_or(0);
                    let mut state = mouse_over_count.lock();
                    if *state == 0 && next > 0 {
                        for handler in &on_mouse_enter {
                            handler(world, id);
                        }
                    }
                    if *state > 0 && next == 0 {
                        for handler in &on_mouse_leave {
                            handler(world, id);
                        }
                    }
                    if next > 0 {
                        for handler in &on_mouse_hover {
                            handler(world, id);
                        }
                    }
                    *state = next;
                }
            }
        });
        hooks.use_world_event({
            let id = id.clone();
            let mouse_over_count = mouse_over_count;
            move |world, event| {
                if let Some(id) = *id.lock() {
                    if let Some(pressed) = event.get(event_mouse_input()) {
                        if *mouse_over_count.lock() > 0 {
                            for handler in &on_mouse_input {
                                handler(world, id, pressed.into(), event.get(mouse_button()).unwrap().into());
                            }
                        }
                    } else if let Some(delta) = event.get(event_mouse_wheel()) {
                        if *mouse_over_count.lock() > 0 {
                            for handler in &on_mouse_wheel {
                                handler(world, id, delta, event.get(event_mouse_wheel_pixels()).unwrap());
                            }
                        }
                    }
                }
            }
        });
        inner.init(mouse_pickable_min(), Vec3::ZERO).init(mouse_pickable_max(), Vec3::ZERO).on_spawned(move |_, new_id| {
            *id.lock() = Some(new_id);
        })
    }
}
