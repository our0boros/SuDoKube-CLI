use crate::GameWorld;
use crate::theme::{ThemeBackground, ThemeColorRole, ThemeColors, ThemeText};
use crate::ui::hud::GameFont;
use bevy::prelude::*;

#[derive(Component)]
pub struct LoadingPanel;

#[derive(Component)]
pub struct LoadingBar;

pub fn spawn_loading_ui(mut commands: Commands, theme: Res<ThemeColors>, font: Res<GameFont>) {
    bevy::log::info!("spawn_loading_ui called");
    let panel = commands
        .spawn((
            GameWorld,
            LoadingPanel,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(theme.background),
            ThemeBackground(ThemeColorRole::Panel),
        ))
        .id();

    let text = commands
        .spawn((
            Text::new("Generating puzzle..."),
            TextFont {
                font_size: 28.0,
                font: font.0.clone(),
                ..default()
            },
            TextColor(theme.text_given),
            ThemeText(ThemeColorRole::TextGiven),
        ))
        .id();
    commands.entity(panel).add_child(text);

    let bar_bg = commands
        .spawn((
            Node {
                width: Val::Px(320.0),
                height: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(theme.panel),
            ThemeBackground(ThemeColorRole::Panel),
        ))
        .id();
    commands.entity(panel).add_child(bar_bg);

    let bar = commands
        .spawn((
            LoadingBar,
            Node {
                width: Val::Px(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(theme.cell_selected),
        ))
        .id();
    commands.entity(bar_bg).add_child(bar);
}

pub fn update_loading_ui(mut bars: Query<&mut Node, With<LoadingBar>>, time: Res<Time>) {
    let t = (time.elapsed_secs().sin() + 1.0) / 2.0;
    for mut node in bars.iter_mut() {
        node.width = Val::Px(320.0 * t);
    }
}

pub fn despawn_loading_ui(
    mut commands: Commands,
    query: Query<Entity, With<LoadingPanel>>,
    children: Query<&Children>,
) {
    for entity in query.iter() {
        despawn_recursive(&mut commands, entity, &children);
    }
}

fn despawn_recursive(commands: &mut Commands, entity: Entity, children: &Query<&Children>) {
    if let Ok(kids) = children.get(entity) {
        for child in kids.iter() {
            despawn_recursive(commands, child, children);
        }
    }
    commands.entity(entity).despawn();
}
