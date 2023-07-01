mod prelude;

use bevy::DefaultPlugins;
use bevy_editor_pls::EditorWindowPlacement;
use bevy_editor_pls::prelude::*;
use crate::prelude::*;

static ENABLE_EDITOR_PLUGIN: bool = false;


fn main() {
    let mut app = App::new();

    if ENABLE_EDITOR_PLUGIN {
        app.add_plugin(EditorPlugin {
            window: EditorWindowPlacement::New(Window {
                title: "Editor".to_string(),
                ..Default::default()
            }),
        });
    }

    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes: true,
        ..Default::default()
    }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            mode: DebugRenderMode::default() | DebugRenderMode::CONTACTS,
            ..default()
        })
        .run();
}
