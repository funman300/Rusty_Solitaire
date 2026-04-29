// Register FontPlugin in solitaire_engine/src/lib.rs before use.

//! Embeds FiraMono-Medium as the project font and exposes it via [`FontResource`].

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

fn load_font(fonts: Option<ResMut<Assets<Font>>>, mut commands: Commands) {
    let Some(mut fonts) = fonts else {
        // Assets<Font> absent (e.g. MinimalPlugins in tests) — insert default.
        commands.insert_resource(FontResource(Handle::default()));
        return;
    };
    let bytes: &'static [u8] = include_bytes!("../../assets/fonts/main.ttf");
    match Font::try_from_bytes(bytes.to_vec()) {
        Ok(font) => {
            commands.insert_resource(FontResource(fonts.add(font)));
        }
        Err(e) => {
            warn!("failed to load main.ttf: {e}; falling back to Bevy default font");
            commands.insert_resource(FontResource(Handle::default()));
        }
    }
}
