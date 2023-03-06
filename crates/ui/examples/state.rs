use std::sync::Arc;

use ambient_app::{App, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_element::{ElementComponent, ElementComponentExt};
use ambient_input::event_mouse_motion;
use ambient_ui::{padding, space_between_items, Borders, Button, Cb, FlowColumn, FlowRow, Text, STREET};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
struct A {
    value: f32,
    set_value: Cb<dyn Fn(f32) + Send + Sync>,
}

impl ElementComponent for A {
    fn render(self: Box<Self>, _: &mut ambient_element::Hooks) -> ambient_element::Element {
        let Self { value, set_value } = *self;
        FlowRow::el([
            Text::el(value.to_string()).set(padding(), Borders::even(STREET)),
            Button::new("+1", {
                let set_value = set_value.clone();
                move |_| set_value(value + 1.0)
            })
            .el(),
            Button::new("-1", move |_| set_value(value - 1.0)).el(),
        ])
        .set(space_between_items(), STREET)
    }
}

#[derive(Debug)]
struct Shared(Arc<String>);

impl Clone for Shared {
    fn clone(&self) -> Self {
        tracing::info!("Cloning {}. Strong: {}", &self.0, Arc::strong_count(&self.0));
        Self(self.0.clone())
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        tracing::info!("Dropping {}. Strong: {}", &self.0, Arc::strong_count(&self.0));
    }
}

pub struct DroppedClosure;
impl Drop for DroppedClosure {
    fn drop(&mut self) {
        tracing::info!("Dropping closure");
    }
}

#[derive(Debug, Clone)]
struct B {
    shared: Shared,
}

impl ElementComponent for B {
    fn render(self: Box<Self>, hooks: &mut ambient_element::Hooks) -> ambient_element::Element {
        let (shared, _) = hooks.use_state(self.shared.clone());
        let keepalive = DroppedClosure;

        hooks.use_world_event(move |_world, event| {
            if let Some(_event) = event.get_ref(event_mouse_motion()) {
                let _val = &keepalive;
            }
        });

        Text::el(shared.0.to_string())
    }
}

impl Drop for B {
    fn drop(&mut self) {
        tracing::info!("Dropping B");
    }
}

#[derive(Debug, Clone)]
struct Main;

impl ElementComponent for Main {
    fn render(self: Box<Self>, hooks: &mut ambient_element::Hooks) -> ambient_element::Element {
        let shared = Shared(Arc::new("Hello, World!".to_string()));

        let (value, set_value) = hooks.use_state(0.0);

        let (show_b, set_show_b) = hooks.use_state(true);
        if show_b {
            FlowColumn::el([A { value, set_value }.el(), Button::new("Hide", move |_| set_show_b(false)).el(), B { shared }.el()])
        } else {
            FlowColumn::el([
                A { value, set_value }.el(),
                Button::new("Show", move |_| set_show_b(true)).el(),
                // B { shared }.el(),
            ])
        }
        .set(space_between_items(), STREET)
    }
}

async fn init(app: &mut App) {
    let world = &mut app.world;
    Main.el().spawn_interactive(world);
    UICamera.el().set(active_camera(), 0.).spawn_interactive(world);
}

fn main() {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    AppBuilder::simple_ui().block_on(init)
}
