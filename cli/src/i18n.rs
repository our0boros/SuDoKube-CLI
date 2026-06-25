/// Internationalization support for SuDoKube

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
    Ja,
}

impl Lang {
    pub fn from_code(code: &str) -> Self {
        match code {
            "zh" => Lang::Zh,
            "ja" => Lang::Ja,
            _ => Lang::En,
        }
    }
}

macro_rules! tr_map {
    ($($key:expr => { zh: $zh:expr, en: $en:expr, ja: $ja:expr $(,)? }),* $(,)?) => {
        pub fn t(key: &str, lang: Lang) -> &'static str {
            match key {
                $($key => match lang {
                    Lang::Zh => $zh,
                    Lang::En => $en,
                    Lang::Ja => $ja,
                },)*
                _ => "???",
            }
        }
    };
}

tr_map! {
    // Menu
    "menu.new_easy" => { zh: "新游戏 - 简单", en: "New Game - Easy", ja: "新ゲーム - 簡単" },
    "menu.new_medium" => { zh: "新游戏 - 中等", en: "New Game - Medium", ja: "新ゲーム - 普通" },
    "menu.new_hard" => { zh: "新游戏 - 困难", en: "New Game - Hard", ja: "新ゲーム - 難しい" },
    "menu.settings" => { zh: "设置", en: "Settings", ja: "設定" },
    "menu.continue" => { zh: "继续", en: "Continue", ja: "続行" },
    "menu.completed" => { zh: "已完成", en: "Done", ja: "完了" },
    "menu.in_progress" => { zh: "进行中", en: "Playing", ja: "プレイ中" },
    "menu.hint_nav" => { zh: "↑/↓ 选择  Enter 确认  D 删除  Q 退出", en: "↑/↓ Select  Enter OK  D Delete  Q Quit", ja: "↑/↓ 選択  Enter 決定  D 削除  Q 終了" },

    // Settings
    "settings.title" => { zh: "设 置", en: "Settings", ja: "設 定" },
    "settings.hint" => { zh: "←/→ 切换  Enter/Esc 返回", en: "←/→ Change  Enter/Esc Back", ja: "←/→ 変更  Enter/Esc 戻る" },

    // Game
    "game.face_front" => { zh: "F前", en: "F-Front", ja: "F-前" },
    "game.face_back" => { zh: "B后", en: "B-Back", ja: "B-後" },
    "game.face_left" => { zh: "L左", en: "L-Left", ja: "L-左" },
    "game.face_right" => { zh: "R右", en: "R-Right", ja: "R-右" },
    "game.face_top" => { zh: "T上", en: "T-Top", ja: "T-上" },
    "game.face_bottom" => { zh: "U下", en: "U-Bottom", ja: "U-下" },
    "game.mode_standard" => { zh: "标准", en: "Std", ja: "標準" },
    "game.mode_mono" => { zh: "等距", en: "Mono", ja: "等幅" },
    "game.diff_easy" => { zh: "简单", en: "Easy", ja: "簡単" },
    "game.diff_medium" => { zh: "中等", en: "Medium", ja: "普通" },
    "game.diff_hard" => { zh: "困难", en: "Hard", ja: "難しい" },

    // Buttons
    "btn.erase" => { zh: "[X]Erase", en: "[X]Erase", ja: "[X]消去" },
    "btn.hint" => { zh: "[H]Hint", en: "[H]Hint", ja: "[H]ﾋﾝﾄ" },
    "btn.undo" => { zh: "[Z]Undo", en: "[Z]Undo", ja: "[Z]戻す" },
    "btn.guide" => { zh: "[G]Guide", en: "[G]Guide", ja: "[G]ｶﾞｲﾄﾞ" },
    "btn.mode" => { zh: "[M]Mode", en: "[M]Mode", ja: "[M]ﾓｰﾄﾞ" },
    "btn.menu" => { zh: "[Q]Menu", en: "[Q]Menu", ja: "[Q]ﾒﾆｭｰ" },

    // Messages
    "msg.generating" => { zh: "正在生成数独立方体...", en: "Generating Sudoku Cube...", ja: "数独立方体を生成中..." },
    "msg.guide_on" => { zh: "辅助模式已开启", en: "Guide mode ON", ja: "ガイドモード ON" },
    "msg.guide_off" => { zh: "辅助模式已关闭", en: "Guide mode OFF", ja: "ガイドモード OFF" },
    "msg.mode_switched" => { zh: "已切换为 {} 模式", en: "Switched to {} mode", ja: "{}モードに切替" },
    "msg.saved" => { zh: "设置已保存", en: "Settings saved", ja: "設定保存済み" },

    // Settings field labels
    "settings.cell_width" => { zh: "标准格宽", en: "Cell Width", ja: "セル幅" },
    "settings.bg_color" => { zh: "背景颜色", en: "BG Color", ja: "背景色" },
    "settings.border_color" => { zh: "边框颜色", en: "Border Color", ja: "枠色" },
    "settings.guide_group" => { zh: "辅助-同组", en: "Guide-Group", ja: "ガイド-同組" },
    "settings.guide_same" => { zh: "辅助-同数", en: "Guide-Same", ja: "ガイド-同数" },
    "settings.cube_scale" => { zh: "Cube缩放", en: "Cube Scale", ja: "Cubeｽｹｰﾙ" },
    "settings.show_cube" => { zh: "显示Cube", en: "Show Cube", ja: "Cube表示" },
    "settings.language" => { zh: "语言", en: "Language", ja: "言語" },
}
