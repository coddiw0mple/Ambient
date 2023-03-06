use ambient_app::{App, AppBuilder};
use ambient_cameras::UICamera;
use ambient_core::camera::active_camera;
use ambient_element::{ElementComponentExt, Group};
use ambient_std::color::Color;
use ambient_ui::*;

async fn init(app: &mut App) {
    let world = &mut app.world;

    let background = |e| FlowRow(vec![e]).el().with_background(Color::rgba(1., 1., 1., 0.02).into());
    Group(vec![
        UICamera.el().set(active_camera(), 0.),
        FlowColumn(vec![
            FlowRow(vec![Text::el("Basic")])
                .el()
                .with_background(Color::rgba(0.1, 0.1, 0.1, 1.).into())
                .set(fit_vertical(), Fit::Children)
                .set(fit_horizontal(), Fit::Children)
                .set(padding(), Borders::even(10.)),
            FlowRow(vec![Text::el("Spacing"), Text::el("between"), Text::el("items")])
                .el()
                .with_background(Color::rgba(0.1, 0.1, 0.1, 1.).into())
                .set(fit_vertical(), Fit::Children)
                .set(fit_horizontal(), Fit::Children)
                .set(padding(), Borders::even(10.))
                .set(space_between_items(), 50.),
            FlowRow(vec![Text::el("Break"), Text::el("line")])
                .el()
                .with_background(Color::rgba(0.1, 0.1, 0.1, 1.).into())
                .set(fit_vertical(), Fit::Children)
                .set(fit_horizontal(), Fit::None)
                .set(width(), 50.)
                .set(padding(), Borders::even(10.)),
            FlowRow(vec![background(Text::el("Align")), background(Text::el("Center").set(font_size(), 30.))])
                .el()
                .with_background(Color::rgba(0.1, 0.1, 0.1, 1.).into())
                .set(fit_vertical(), Fit::None)
                .set(fit_horizontal(), Fit::None)
                .set(align_horizontal(), Align::Center)
                .set(align_vertical(), Align::Center)
                .set(width(), 200.)
                .set(height(), 70.)
                .set(padding(), Borders::even(10.))
                .set(space_between_items(), 5.),
            FlowRow(vec![background(Text::el("Align")), background(Text::el("End").set(font_size(), 30.))])
                .el()
                .with_background(Color::rgba(0.1, 0.1, 0.1, 1.).into())
                .set(fit_vertical(), Fit::None)
                .set(fit_horizontal(), Fit::None)
                .set(align_horizontal(), Align::End)
                .set(align_vertical(), Align::End)
                .set(width(), 200.)
                .set(height(), 70.)
                .set(padding(), Borders::even(10.))
                .set(space_between_items(), 5.),
        ])
        .el()
        .set(space_between_items(), 5.)
        .set(padding(), Borders::even(5.)),
    ])
    .el()
    .spawn_interactive(world);
}

fn main() {
    env_logger::init();
    AppBuilder::simple_ui().block_on(init);
}
