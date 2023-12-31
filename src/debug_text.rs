use crate::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use crate::camera_control::MouseTargetedEntity;

pub struct DebugTextPlugin;

impl Plugin for DebugTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (fps_update_system, mouse_over_target));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let fira_sans_bold = asset_server.load("fonts/FiraSans-Bold.ttf");
    let fira_mono_medium = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: fira_sans_bold.clone(),
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: fira_mono_medium.clone(),
                font_size: 16.0,
                color: Color::WHITE,
            }),
        ])
        .with_background_color(Color::BLACK.with_a(0.5))
            .with_style(Style {
                position_type: PositionType::Absolute,
                ..default()
            }),
        FpsText,
    ));

    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "Target: ",
                TextStyle {
                    font: fira_sans_bold,
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: fira_mono_medium,
                font_size: 16.0,
                color: Color::WHITE,
            }),
        ])
        .with_background_color(Color::BLACK.with_a(0.5))
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                ..default()
            }),
        MouseOverText,
    ));

}

#[derive(Component)]
struct FpsText;

fn fps_update_system(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

#[derive(Component)]
struct MouseOverText;

fn mouse_over_target(target: Res<MouseTargetedEntity>, mut query: Query<&mut Text, With<MouseOverText>>, entity_info: Query<(Option<&Name>, Option<&GlobalTransform>)>) {
    for mut text in &mut query {

        if let Some(target) = &target.target {

            match entity_info.get(target.entity) {
                Ok((Some(name), Some(transform))) => {
                    text.sections[1].value = format!("{}: {:?}, {:?}", name, transform.translation(), target.intersection.point);
                },
                _ => {
                    text.sections[1].value = format!("{:?}: {:?}", target.entity, target.intersection.point);
                }
            }
        } else {
            text.sections[1].value = "None".to_string();
        }

    }
}
