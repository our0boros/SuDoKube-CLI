use crate::app_state::AppState;
use crate::theme::{ThemeBackground, ThemeBorder, ThemeColorRole, ThemeColors, ThemeText};
use crate::ui::hud::GameFont;
use bevy::prelude::*;
use crate::save::db::{GameRecord, load_history};

#[derive(Component)]
pub struct HistoryPanel;

#[derive(Component)]
pub struct HistoryEntryText;

#[derive(Component)]
pub enum HistoryButton {
    Close,
}

pub fn toggle_history(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::InGame => next_state.set(AppState::History),
            AppState::History => next_state.set(AppState::InGame),
            _ => {}
        }
    }
}

pub fn spawn_history_panel(
    mut commands: Commands,
    theme: Res<ThemeColors>,
    font: Res<GameFont>,
    query: Query<Entity, With<HistoryPanel>>,
) {
    if !query.is_empty() {
        return;
    }

    let records = load_history(50).unwrap_or_default();

    let panel = commands
        .spawn((
            HistoryPanel,
            Node {
                width: Val::Px(520.0),
                height: Val::Percent(80.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect::new(Val::Px(-260.0), Val::Auto, Val::Percent(-40.0), Val::Auto),
                ..default()
            },
            BackgroundColor(theme.panel),
            ThemeBackground(ThemeColorRole::Panel),
            BorderColor::all(theme.panel_border),
            ThemeBorder(ThemeColorRole::PanelBorder),
            Visibility::Hidden,
        ))
        .id();

    let inner = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },))
        .id();
    commands.entity(panel).add_child(inner);

    let header = commands
        .spawn((
            Text::new("History (ESC to close)"),
            TextFont {
                font_size: 24.0,
                font: font.0.clone(),
                ..default()
            },
            TextColor(theme.text_given),
            ThemeText(ThemeColorRole::TextGiven),
        ))
        .id();
    commands.entity(inner).add_child(header);

    let close_btn = commands
        .spawn((
            Button,
            HistoryButton::Close,
            Node {
                width: Val::Px(80.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme.button),
            ThemeBackground(ThemeColorRole::Button),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Close"),
                TextFont {
                    font_size: 16.0,
                    font: font.0.clone(),
                    ..default()
                },
                TextColor(theme.button_text),
                ThemeText(ThemeColorRole::ButtonText),
            ));
        })
        .id();
    commands.entity(inner).add_child(close_btn);

    if records.is_empty() {
        let empty = commands
            .spawn((
                Text::new("No history yet"),
                TextFont {
                    font_size: 18.0,
                    font: font.0.clone(),
                    ..default()
                },
                TextColor(theme.text_note),
                ThemeText(ThemeColorRole::ButtonText),
            ))
            .id();
        commands.entity(inner).add_child(empty);
    } else {
        for record in records {
            let line = format_history_line(&record);
            let text = commands
                .spawn((
                    Text::new(line),
                    TextFont {
                        font_size: 16.0,
                        font: font.0.clone(),
                        ..default()
                    },
                    TextColor(theme.text_given),
                    ThemeText(ThemeColorRole::TextGiven),
                    HistoryEntryText,
                ))
                .id();
            commands.entity(inner).add_child(text);
        }
    }
}

pub fn update_history_visibility(
    state: Res<State<AppState>>,
    mut panels: Query<&mut Visibility, With<HistoryPanel>>,
) {
    let visible = *state.get() == AppState::History;
    for mut v in panels.iter_mut() {
        *v = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn handle_history_buttons(
    interaction_query: Query<(&Interaction, &HistoryButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, button) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                HistoryButton::Close => next_state.set(AppState::InGame),
            }
        }
    }
}

fn format_history_line(record: &GameRecord) -> String {
    let status = if record.completed { "Done" } else { "Open" };
    format!(
        "{} | {} | {}s | {}",
        record.started_at.format("%m-%d %H:%M"),
        record.difficulty,
        record.elapsed_seconds,
        status
    )
}
