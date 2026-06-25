use crate::app_state::AppState;
use crate::theme::{ThemeBackground, ThemeBorder, ThemeColorRole, ThemeColors, ThemeText};
use bevy::prelude::*;
use sudokube_core::game_state::GameState;
use sudokube_core::theme::Theme;

#[derive(Resource)]
pub struct GameFont(pub Handle<Font>);

#[derive(Component, Debug, Clone, Copy)]
pub enum HudButton {
    Number(u8),
    Hint,
    Undo,
    GiveUp,
    AutoNext,
    ThemeToggle,
    NewGame,
}

pub fn spawn_hud(
    mut commands: Commands,
    theme: Res<ThemeColors>,
    font: Res<GameFont>,
    roots: Query<Entity, With<HudRoot>>,
) {
    if !roots.is_empty() {
        return;
    }

    let root = commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();

    let top_bar = spawn_top_bar(&mut commands, &theme, &font.0);
    commands.entity(root).add_child(top_bar);

    let side_bar = spawn_side_bar(&mut commands, &theme, &font.0);
    commands.entity(root).add_child(side_bar);

    let bottom_spacer = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(8.0),
            ..default()
        },))
        .id();
    commands.entity(root).add_child(bottom_spacer);
}

fn spawn_top_bar(commands: &mut Commands, theme: &ThemeColors, font: &Handle<Font>) -> Entity {
    let bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(56.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(theme.panel),
            ThemeBackground(ThemeColorRole::Panel),
        ))
        .id();

    let title = spawn_text(commands, "SuDoKube", 28.0, ThemeColorRole::TextGiven, font);
    let timer = spawn_text(commands, "00:00", 24.0, ThemeColorRole::ButtonText, font);
    commands.entity(timer).insert(TimerText);

    let buttons = commands
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            ..default()
        },))
        .id();

    commands.entity(bar).add_child(title);
    commands.entity(bar).add_child(timer);
    commands.entity(bar).add_child(buttons);

    let theme_btn = spawn_button(commands, "Theme", theme, font);
    commands.entity(theme_btn).insert(HudButton::ThemeToggle);
    commands.entity(buttons).add_child(theme_btn);

    bar
}

fn spawn_side_bar(commands: &mut Commands, theme: &ThemeColors, font: &Handle<Font>) -> Entity {
    let bar = commands
        .spawn((
            Node {
                width: Val::Px(120.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(56.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(theme.panel),
            ThemeBackground(ThemeColorRole::Panel),
        ))
        .id();

    let num_grid = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(6.0),
            column_gap: Val::Px(6.0),
            ..default()
        },))
        .id();
    commands.entity(bar).add_child(num_grid);

    for n in 1..=9u8 {
        let btn = spawn_button(commands, &n.to_string(), theme, font);
        commands.entity(btn).insert(HudButton::Number(n));
        commands.entity(num_grid).add_child(btn);
    }

    for (label, action) in [
        ("Hint", HudButton::Hint),
        ("Undo", HudButton::Undo),
        ("Give Up", HudButton::GiveUp),
        ("Auto Next", HudButton::AutoNext),
        ("New Game", HudButton::NewGame),
    ] {
        let btn = spawn_button(commands, label, theme, font);
        commands.entity(btn).insert(action);
        commands.entity(bar).add_child(btn);
    }

    bar
}

fn spawn_button(
    commands: &mut Commands,
    label: &str,
    theme: &ThemeColors,
    font: &Handle<Font>,
) -> Entity {
    commands
        .spawn((
            Button,
            Node {
                width: Val::Px(48.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(theme.button),
            ThemeBackground(ThemeColorRole::Button),
            BorderColor::all(theme.panel_border),
            ThemeBorder(ThemeColorRole::PanelBorder),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    font: font.clone(),
                    ..default()
                },
                TextColor(theme.button_text),
                ThemeText(ThemeColorRole::ButtonText),
            ));
        })
        .id()
}

fn spawn_text(
    commands: &mut Commands,
    text: &str,
    size: f32,
    role: ThemeColorRole,
    font: &Handle<Font>,
) -> Entity {
    commands
        .spawn((
            Text::new(text),
            TextFont {
                font_size: size,
                font: font.clone(),
                ..default()
            },
            TextColor(ThemeColors::default().color(role)),
            ThemeText(role),
        ))
        .id()
}

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct TimerText;

pub fn update_timer(
    mut texts: Query<&mut Text, With<TimerText>>,
    game_state: Res<GameState>,
    time: Res<Time>,
) {
    if game_state.started_at > 0.0 && !game_state.completed {
        let elapsed = time.elapsed_secs_f64() - game_state.started_at;
        let minutes = (elapsed / 60.0) as u64;
        let seconds = (elapsed % 60.0) as u64;
        for mut text in texts.iter_mut() {
            text.0 = format!("{:02}:{:02}", minutes, seconds);
        }
    }
}

pub fn handle_hud_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ThemeBackground,
            &HudButton,
        ),
        Changed<Interaction>,
    >,
    mut game_state: ResMut<GameState>,
    mut theme: ResMut<Theme>,
    theme_colors: Res<ThemeColors>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut bg, theme_bg, button) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                *bg = BackgroundColor(theme_colors.button_hover);
            }
            Interaction::None => {
                *bg = BackgroundColor(theme_colors.color(theme_bg.0));
            }
            Interaction::Pressed => match *button {
                HudButton::Number(n) => {
                    if let Some(coord) = game_state.selected {
                        game_state.set_value(coord, Some(n));
                    }
                }
                HudButton::Hint => game_state.hint(),
                HudButton::Undo => game_state.undo(),
                HudButton::GiveUp => game_state.give_up(),
                HudButton::AutoNext => game_state.auto_next = !game_state.auto_next,
                HudButton::ThemeToggle => *theme = theme.toggle(),
                HudButton::NewGame => next_state.set(AppState::Loading),
            },
        }
    }
}
