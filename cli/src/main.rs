//! SuDoKube CLI — 主入口

mod app;
mod config;
mod game_utils;
mod i18n;
mod input;
mod menu;
mod render;
mod save;
mod settings;
mod shop;
mod types;
mod widgets;

// 公共重导出（保持其他模块通过 crate:: 引用不变）
pub use app::App;
pub use game_utils::{continue_game, current_coord, flush_elapsed, now_secs, total_elapsed};
pub use menu::{MenuItem, MenuState};
pub use settings::{AppSettings, SettingsField, SettingsState};
pub use types::{AppScreen, GeneratingState, KeymapEditState, PositionKind, SettingsArrow, position_kind};
pub use widgets::{
    Button, ButtonState, ButtonTheme, EdgeDecor, THEME_PRIMARY, THEME_SUCCESS, THEME_DANGER,
    THEME_NEUTRAL,
};

use std::io;
use std::time::{Duration, Instant};

use crossterm::event;
use input::{EventResult, handle_event};

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture)?;
    let result = run_app(terminal);
    crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture)?;
    ratatui::restore();
    result
}

fn run_app(
    mut terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
) -> io::Result<()> {
    let mut app = App::new_menu();
    let mut last_blink = Instant::now();

    loop {
        // 闪烁定时器(仅在设置开启时切换)
        let now = Instant::now();
        if app.settings.blink_highlight == "on"
            && now.duration_since(last_blink) >= Duration::from_millis(500)
        {
            app.blink_on = !app.blink_on;
            last_blink = now;
        }
        app.clear_message_if_expired();

        // 3D立方体自动旋转（两轴不同周期）
        app.cube_angle_y += 0.03;
        app.cube_angle_x += 0.02;

        // 胜利倒计时
        if app.screen == AppScreen::Victory {
            if let Some(until) = app.victory_countdown {
                if Instant::now() >= until {
                    app = App::new_menu();
                }
            }
        }

        // 检查生成进度
        if app.screen == AppScreen::Generating {
            if let Some(game) = app.check_generating() {
                app.generating = None;
                app = App::start_game(game);
            } else if let Some(ref mut gen_state) = app.generating {
                gen_state.spinner = (gen_state.spinner + 1) % 4;
            }
        }

        // 贪吃蛇: 每帧推进 + 胜负结算时退出
        if app.snake.is_some() {
            shop::snake_step(&mut app);
            if let Some(snake) = app.snake.as_ref() {
                if snake.outcome != shop::SnakeOutcome::Running {
                    if let Some(msg) = shop::end_snake_game(&mut app) {
                        app.set_message(msg, Duration::from_secs(2));
                    }
                }
            }
        }

        // 渲染
        if app.screen == AppScreen::Game {
            let area: ratatui::layout::Rect = terminal.size()?.into();
            let needs_overflow_switch = render::needs_scrollbar_mode(area, &app);
            if needs_overflow_switch && app.render_mode != render::RenderMode::Scrollbar {
                let prev = app.render_mode;
                app.render_mode = render::RenderMode::Scrollbar;
                if prev != render::RenderMode::Scrollbar && app.overflow_notice_elapsed.is_none()
                {
                    let lang = crate::i18n::Lang::from_code(&app.settings.language);
                    let label = render::mode_label(app.render_mode, lang);
                    app.set_message(
                        format!("⚠ {} ({})", i18n::t("msg.overflow_auto", lang), label),
                        Duration::from_secs(3),
                    );
                    app.overflow_notice_elapsed = Some(Instant::now() + Duration::from_secs(3));
                }
            }
        }

        terminal.draw(|f| render::draw(f, &mut app))?;

        // 事件处理
        if event::poll(Duration::from_millis(50))? {
            let ev = event::read()?;
            let area: ratatui::layout::Rect = terminal.size()?.into();
            match handle_event(&mut app, ev, area) {
                EventResult::Continue => {}
                EventResult::StartGame(game) => {
                    app = App::start_game(game);
                }
                EventResult::StartGenerating(difficulty) => {
                    app.start_generating(difficulty);
                }
                EventResult::BackToMenu => {
                    if app.screen == AppScreen::Game {
                        flush_elapsed(&mut app);
                        let _ = save::save_game(&app.game);
                    }
                    app = App::new_menu();
                }
                EventResult::Quit => {
                    if app.screen == AppScreen::Game {
                        flush_elapsed(&mut app);
                        let _ = save::save_game(&app.game);
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}
