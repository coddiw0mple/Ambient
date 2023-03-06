use ambient_app::{App, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_element::{ElementComponentExt, Group};
use ambient_std::color::Color;
use ambient_ui::*;

async fn init(app: &mut App) {
    let world = &mut app.world;
    let card_inner =
        |text| FlowRow(vec![Text::el(text)]).el().with_background(Color::rgba(0.3, 0.3, 0.3, 1.).into()).set(padding(), Borders::even(20.));
    Group(vec![
        UICamera.el().set(active_camera(), 0.),
        FlowRow(vec![
            FlowColumn(vec![
                Button::new("Regular", |_| {}).el(),
                Button::new("Primary", |_| {}).style(ButtonStyle::Primary).tooltip(Text::el("Tooltip")).el(),
                Button::new("Flat", |_| {}).style(ButtonStyle::Flat).el(),
                Button::new(card_inner("Card"), |_| {}).style(ButtonStyle::Card).el(),
                Button::new("Inline", |_| {}).style(ButtonStyle::Inline).el(),
            ])
            .el()
            .set(space_between_items(), STREET)
            .set(padding(), Borders::even(STREET)),
            FlowColumn(vec![
                Button::new("Regular toggled", |_| {}).toggled(true).el(),
                Button::new("Primary toggled", |_| {}).toggled(true).style(ButtonStyle::Primary).el(),
                Button::new("Flat toggled", |_| {}).toggled(true).style(ButtonStyle::Flat).el(),
                Button::new(card_inner("Card toggled"), |_| {}).toggled(true).style(ButtonStyle::Card).el(),
                Button::new("Inline toggled", |_| {}).toggled(true).style(ButtonStyle::Inline).el(),
            ])
            .el()
            .set(space_between_items(), STREET)
            .set(padding(), Borders::even(STREET)),
            FlowColumn(vec![
                Button::new("Regular disabled", |_| {}).disabled(true).el(),
                Button::new("Primary disabled", |_| {}).disabled(true).style(ButtonStyle::Primary).el(),
                Button::new("Flat disabled", |_| {}).disabled(true).style(ButtonStyle::Flat).el(),
                Button::new(card_inner("Card disabled"), |_| {}).disabled(true).style(ButtonStyle::Card).el(),
                Button::new("Inline disabled", |_| {}).disabled(true).style(ButtonStyle::Inline).el(),
            ])
            .el()
            .set(space_between_items(), STREET)
            .set(padding(), Borders::even(STREET)),
            Button::new(FontAwesomeIcon { icon: 0xf1e2, solid: true }.el(), |_| {}).el(),
        ])
        .el()
        .set(space_between_items(), STREET),
    ])
    .el()
    .spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().block_on(init);
}
