use ambient_core::{
    hierarchy::children,
    transform::{local_to_world, translation},
};
use ambient_ecs::{components, query, SystemGroup};
use ambient_element::{define_el_function_for_vec_element_newtype, Element, ElementComponent, ElementComponentExt, Hooks};
use glam::vec3;

use crate::{app_background_color, padding, Borders, Dock, UIBase, WindowSized};
use ambient_ui_components::UIExt;

components!("ui", {
    screen: (),
});

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "ui/screens",
        vec![query((local_to_world().changed(), children().changed())).incl(screen()).to_system(|q, world, qs, _| {
            for (_, (ltw, children)) in q.collect_cloned(world, qs) {
                let (_, _, pos) = ltw.to_scale_rotation_translation();
                for c in children {
                    if let Ok(p) = world.get_mut(c, translation()) {
                        p.x = -pos.x;
                        p.y = -pos.y;
                    }
                }
            }
        })],
    )
}

#[derive(Clone, Debug)]
pub struct ScreenContainer(pub Option<Element>);
impl ElementComponent for ScreenContainer {
    #[allow(clippy::clone_on_copy)]
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        if let Some(content) = self.0 {
            UIBase.el().set(screen(), ()).children(vec![WindowSized(vec![Dock(vec![content]).el().set(translation(), vec3(0., 0., 0.1))])
                .el()
                .with_background(app_background_color().set_a(0.99).clone().into())
                .with_clickarea()
                .el()])
        } else {
            Element::new()
        }
    }
}

#[derive(Clone, Debug)]
pub struct PageScreen(pub Vec<Element>);
define_el_function_for_vec_element_newtype!(PageScreen);
impl ElementComponent for PageScreen {
    #[allow(clippy::clone_on_copy)]
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        WindowSized(vec![Dock(self.0).el().init(padding(), Borders::even(30.))])
            .el()
            .with_background(app_background_color().set_a(0.99).clone().into())
            .with_clickarea()
            .el()
    }
}

#[derive(Clone, Debug)]
pub struct DialogScreen(pub Element);
impl ElementComponent for DialogScreen {
    #[allow(clippy::clone_on_copy)]
    fn render(self: Box<Self>, _: &mut Hooks) -> Element {
        WindowSized(vec![Dock(vec![self.0]).el().init(padding(), Borders::even(30.))])
            .el()
            .with_background(app_background_color().set_a(0.99).clone().into())
            .with_clickarea()
            .el()
    }
}
