use crate::render::RenderPlugin;
use bevy::app::App;
use bevy::DefaultPlugins;

mod render;

fn main() {
    let mut builder = App::build();

    builder.add_plugins(DefaultPlugins).add_plugin(RenderPlugin);

    builder.run();
}
