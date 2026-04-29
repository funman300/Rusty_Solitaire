// Register FontPlugin in solitaire_engine/src/lib.rs before use.

//! Loads FiraMono-Medium via the Bevy `AssetServer` and exposes it via [`FontResource`].

use bevy::prelude::*;

/// Holds the project-wide [`Handle<Font>`] loaded at startup.
#[derive(Resource)]
pub struct FontResource(pub Handle<Font>);

/// Loads FiraMono-Medium at startup and inserts [`FontResource`].
pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_font);
    }
}

fn load_font(asset_server: Option<Res<AssetServer>>, mut commands: Commands) {
    let Some(asset_server) = asset_server else {
        // AssetServer absent (e.g. MinimalPlugins in tests) — insert default.
        commands.insert_resource(FontResource(Handle::default()));
        return;
    };
    commands.insert_resource(FontResource(asset_server.load("fonts/main.ttf")));
}
