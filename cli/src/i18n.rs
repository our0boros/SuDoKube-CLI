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
    "settings.cube_width" => { zh: "Cube宽度", en: "Cube Width", ja: "Cube幅" },
    "settings.cube_height" => { zh: "Cube高度", en: "Cube Height", ja: "Cube高さ" },
    "settings.debug_mode" => { zh: "调试模式", en: "Debug Mode", ja: "ﾃﾞﾊﾞｯｸﾞ" },
    "settings.language" => { zh: "语言", en: "Language", ja: "言語" },

    // Debug
    "debug.hint" => { zh: "已提示", en: "Hinted", ja: "ﾋﾝﾄ済み" },

    // Info bar
    "info.face" => { zh: "面", en: "Face", ja: "面" },
    "info.diff" => { zh: "难度", en: "Diff", ja: "難度" },
    "info.time" => { zh: "时间", en: "Time", ja: "時間" },
    "game.unnamed" => { zh: "新对局", en: "New", ja: "新規" },

    // Victory
    "victory.title" => { zh: "恭喜通关!", en: "Victory!", ja: "クリア!" },
    "victory.subtitle" => { zh: "你赢了 ヾ(≧▽≦*)o", en: "You won! (≧▽≦)", ja: "クリア! ヾ(≧▽≦)o" },
    "victory.countdown" => { zh: "自动返回 {}s", en: "Auto back {}s", ja: "自動戻り {}s" },
    "victory.enter" => { zh: "按 Enter 返回", en: "Press Enter to return", ja: "Enterで戻る" },

    // Menu extra
    "menu.export" => { zh: "导出对局", en: "Export Game", ja: "対局ｴｸｽﾎﾟｰﾄ" },
    "menu.import" => { zh: "导入对局", en: "Import Game", ja: "対局ｲﾝﾎﾟｰﾄ" },

    // Export
    "export.encrypted" => { zh: "加密导出", en: "Encrypted Export", ja: "暗号化ｴｸｽﾎﾟｰﾄ" },
    "export.plaintext" => { zh: "明文导出", en: "Plaintext Export", ja: "平文ｴｸｽﾎﾟｰﾄ" },
    "export.copied" => { zh: "已复制到剪贴板", en: "Copied to clipboard", ja: "ｸﾘｯﾌﾟﾎﾞｰﾄﾞにｺﾋﾟｰ" },
    "import.paste" => { zh: "请粘贴对局数据后按 Enter", en: "Paste game data then Enter", ja: "対局ﾃﾞｰﾀを貼付けてEnter" },
    "import.success" => { zh: "导入成功!", en: "Import successful!", ja: "ｲﾝﾎﾟｰﾄ成功!" },
    "import.fail" => { zh: "导入失败: 无效数据", en: "Import failed: invalid data", ja: "ｲﾝﾎﾟｰﾄ失敗: 無効ﾃﾞｰﾀ" },

    // Naming mode
    "naming.vivid" => { zh: "生动", en: "Vivid", ja: "鮮やか" },
    "naming.numeric" => { zh: "数字", en: "Numeric", ja: "数字" },
}

/// Adjective names for vivid naming mode
pub fn adjectives(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::Zh => &["兴奋的", "快乐的", "安静的", "勇敢的", "聪明的", "温柔的", "活泼的", "优雅的", "坚定的", "灵巧的",
                      "热情的", "悠闲的", "可爱的", "神秘的", "淡定的", "潇洒的", "顽强的", "机智的", "沉着的", "霸气的"],
        Lang::En => &["Excited", "Happy", "Quiet", "Brave", "Clever", "Gentle", "Lively", "Elegant", "Steady", "Nimble",
                      "Warm", "Relaxed", "Cute", "Mysterious", "Calm", "Cool", "Tough", "Witty", "Poised", "Bold"],
        Lang::Ja => &["興奮の", "楽しい", "静かな", "勇敢な", "聡明な", "優しい", "活発な", "優雅な", "堅実な", "機敏な",
                      "情熱の", "のんきな", "かわいい", "神秘の", "冷静な", "かっこいい", "しぶとい", "機知な", "落ち着いた", "豪快な"],
    }
}

/// Animal names for vivid naming mode
pub fn animals(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::Zh => &["熊猫", "狐狸", "兔子", "老虎", "海豚", "猫头鹰", "松鼠", "企鹅", "鹿", "鲸鱼",
                      "小猫", "柴犬", "鹦鹉", "海龟", "刺猬", "水獭", "浣熊", "考拉", "火烈鸟", "独角兽"],
        Lang::En => &["Panda", "Fox", "Rabbit", "Tiger", "Dolphin", "Owl", "Squirrel", "Penguin", "Deer", "Whale",
                      "Cat", "Shiba", "Parrot", "Turtle", "Hedgehog", "Otter", "Raccoon", "Koala", "Flamingo", "Unicorn"],
        Lang::Ja => &["パンダ", "キツネ", "ウサギ", "トラ", "イルカ", "フクロウ", "リス", "ペンギン", "シカ", "クジラ",
                      "ネコ", "シバ", "オウム", "カメ", "ハリネズミ", "カワウソ", "アライグマ", "コアラ", "フラミンゴ", "ユニコーン"],
    }
}

/// Generate a vivid name like "兴奋的熊猫#1"
pub fn vivid_name(id: i64, lang: Lang) -> String {
    let adjs = adjectives(lang);
    let anms = animals(lang);
    let adj = adjs[(id as usize).wrapping_rem(adjs.len())];
    let anm = anms[(id as usize).wrapping_rem(anms.len())];
    format!("{}{}#{}", adj, anm, id)
}
