mod logging;
mod css;
mod model;
mod licenses;
mod db;
mod import;

use dioxus::document;
use dioxus::prelude::*;
use db::Database;
use model::{sleep_ms, Deck, FsrsConfig, FsrsRating, QuizState, Screen, Word};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use import::*;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Debug)]
enum ThemeMode { System, Light, Dark }

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const GH_REPO: &str = "HongyiHank/scallion-vocab";
const MAX_RECENT_URLS: usize = 5;
const TOAST_DURATION_MS: u64 = 2_800;
const DEFAULT_AUTO_ADVANCE_MS: i64 = 1_000;
const ANTI_FOUC_SCRIPT: &str = "try{var t=localStorage.getItem('theme')||'system';if(t==='system'){try{t=AndroidSystemTheme.isSystemDark()?'dark':'light'}catch(e){t='light'}}document.documentElement.setAttribute('data-theme',t)}catch(_){}";

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct UpdateInfo {
    tag: String,
    url: String,
    size: u64,
}

fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let v = v.strip_prefix('v').unwrap_or(v);
    let parts: Vec<&str> = v.split('.').collect();
    Some((
        parts.first()?.parse().ok()?,
        parts.get(1)?.parse().ok()?,
        parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
    ))
}

#[derive(Clone, Debug, PartialEq)]
struct ToastState {
    id: u64,
    text: String,
}

#[derive(Clone, Debug)]
struct ExamPendingName {
    pub names: String,
    pub word_count: usize,
}

#[derive(Clone, Copy)]
struct AppSignals {
    screen: Signal<Screen>,
    quiz: Signal<Option<QuizState>>,
    toast: Signal<Option<ToastState>>,
    toast_seq: Signal<u64>,
    theme_mode: Signal<ThemeMode>,
    is_dark: Signal<bool>,  // resolved dark mode (derived from theme_mode + system detection)
    infinite_mode: Signal<bool>,
    show_finished_screen: Signal<bool>,
    auto_advance_ms: Signal<i64>,
    fsrs_config: Signal<FsrsConfig>,
    prefs_loaded: Signal<bool>,
    recent_urls: Signal<Vec<String>>,
    update_info: Signal<Option<UpdateInfo>>,
    download_progress: Signal<Option<f64>>,
    show_reset_confirm: Signal<bool>,
    update_check_enabled: Signal<bool>,
    db: Signal<Option<Database>>,
    exam_pending_name: Signal<Option<ExamPendingName>>,
}

fn push_toast(mut app: AppSignals, msg: impl Into<String>) {
    let id = (*app.toast_seq.read()).wrapping_add(1);
    app.toast_seq.set(id);
    app.toast.set(Some(ToastState {
        id,
        text: msg.into(),
    }));
}

#[component]
fn ModalDialog(visible: bool, title: String, children: Element) -> Element {
    if !visible {
        return VNode::empty();
    }
    rsx! {
        div { class: "update-overlay",
            div { class: "update-dialog",
                div { class: "update-title", "{title}" }
                {children}
            }
        }
    }
}

#[allow(non_snake_case)]
fn App() -> Element {
    use_context_provider(|| AppSignals {
        screen: Signal::new(Screen::Exam),
        quiz: Signal::new(None),
        toast: Signal::new(None),
        toast_seq: Signal::new(0),
        theme_mode: Signal::new(ThemeMode::System),
        is_dark: Signal::new(false),
        infinite_mode: Signal::new(true),
        show_finished_screen: Signal::new(true),
        auto_advance_ms: Signal::new(DEFAULT_AUTO_ADVANCE_MS),
        fsrs_config: Signal::new(FsrsConfig::default()),
        prefs_loaded: Signal::new(false),
        recent_urls: Signal::new(Vec::new()),
        update_info: Signal::new(None),
        download_progress: Signal::new(None),
        show_reset_confirm: Signal::new(false),
        update_check_enabled: Signal::new(true),
        db: Signal::new(None),
        exam_pending_name: Signal::new(None),
    });

    let mut app = use_context::<AppSignals>();

    use_effect(move || {
        spawn(async move {
            load_prefs(app).await;
        });
    });

    // 主題模式 → 解析 is_dark + 持久化
    use_effect(move || {
        if !*app.prefs_loaded.read() {
            return;
        }

        let mode = *app.theme_mode.read();

        spawn(async move {
            let is_dark = match mode {
                ThemeMode::Light => false,
                ThemeMode::Dark => true,
                ThemeMode::System => {
                    let mut eval = document::eval(
                        r#"try { dioxus.send(AndroidSystemTheme.isSystemDark() ? 'true' : 'false'); } catch(_) { dioxus.send('false'); }"#,
                    );
                    eval.recv::<String>().await.map(|s| s == "true").unwrap_or(false)
                }
            };
            app.is_dark.set(is_dark);
            persist_theme(mode, is_dark).await;
        });
    });

    // 其他信號持久化（非主題）
    use_effect(move || {
        if !*app.prefs_loaded.read() {
            return;
        }

        let _ = *app.is_dark.read();  // 追蹤以重跑
        let infinite = *app.infinite_mode.read();
        let show_finished = *app.show_finished_screen.read();
        let cfg = app.fsrs_config.cloned();
        let ms = *app.auto_advance_ms.read();
        let uc = *app.update_check_enabled.read();

        spawn(async move {
            persist_infinite_mode(infinite).await;
            persist_show_finished_screen(show_finished).await;
            persist_fsrs_config(cfg).await;
            persist_auto_advance_ms(ms).await;
            persist_update_check_enabled(uc).await;
        });
    });

    use_effect(move || {
        let _ = *app.toast_seq.read();
        let Some(id) = app.toast.read().as_ref().map(|t| t.id) else {
            return;
        };
        spawn(async move {
            sleep_ms(TOAST_DURATION_MS).await;
            let mut guard = app.toast.write();
            if guard.as_ref().map(|t| t.id) == Some(id) {
                *guard = None;
            }
        });
    });

    use_effect(move || {
        spawn(async move {
            if app.db.read().is_some() {
                return;
            }
            match Database::open() {
                Ok(database) => {
                    log!("[DB::Init] database opened successfully");
                    app.db.set(Some(database));
                }
                Err(e) => {
                    log!("[DB::Init] failed to open database: {e}");
                }
            }
        });
    });

    // Check for updates on launch (after prefs loaded, if enabled).
    use_effect(move || {
        if !*app.prefs_loaded.read() || !*app.update_check_enabled.read() {
            return;
        }
        let mut app_clone = app;
        spawn(async move {
            let js = format!(
                    r#"var sv=JSON.parse(localStorage.getItem('skipped_versions')||'[]');fetch('https://api.github.com/repos/{repo}/releases/latest',{{headers:{{'Accept':'application/json','User-Agent':'scallion-vocab'}}}}).then(r=>r.json()).then(d=>{{var tag=d.tag_name||'';if(sv.includes(tag)){{dioxus.send('');return;}}var info=JSON.stringify({{tag:tag,url:(d.assets&&d.assets[0])?d.assets[0].browser_download_url:'',size:(d.assets&&d.assets[0])?d.assets[0].size:0}});dioxus.send(info)}}).catch(function(){{dioxus.send('')}});"#,
                    repo = GH_REPO
            );
            let mut eval = document::eval(&js);
            match eval.recv::<String>().await {
                Ok(json) if !json.is_empty() => {
                    if let Ok(info) = serde_json::from_str::<UpdateInfo>(&json) {
                        if !info.tag.is_empty()
                            && !info.url.is_empty()
                            && parse_version(&info.tag).map_or(false, |v| {
                                parse_version(APP_VERSION).map_or(true, |cur| v > cur)
                            })
                        {
                            app_clone.update_info.set(Some(info));
                        }
                    }
                }
                _ => {}
            }
        });
    });

    // Register global JS handler for Android hardware back button.
    // Called from MainActivity.onKeyDown before any default WebView/Activity back behaviour.
    use_effect(move || {
        spawn(async move {
            let js = r#"
            window.__handleAndroidBack = function() {
                var el;
                // License detail overlay → back to list
                el = document.querySelector('.license-detail-close');
                if (el) { el.click(); return; }
                // License list → close
                el = document.querySelector('.license-dialog-close');
                if (el) { el.click(); return; }
                // Settings/Import screen → back to Upload
                el = document.querySelector('.settings-close');
                if (el) { el.click(); return; }
                el = document.querySelector('.import-back');
                if (el) { el.click(); return; }
                // Deck detail screen → back to Library
                el = document.querySelector('.deck-detail-back');
                if (el) { el.click(); return; }
                // Library screen: 委派給 .library-back 處理（回上層或根目錄時觸發「再按一次以退出」）
                el = document.querySelector('.library-back');
                if (el) { el.click(); return; }
                // Exam screen history panel open → close it
                el = document.querySelector('.history-overlay.open');
                if (el) { el.click(); return; }
                // Quiz screen → show pause overlay (top-right icon button)
                el = document.querySelector('.quiz-screen');
                if (el) { var b = el.querySelector('.top-icon-btn'); if (b) { b.click(); return; } }
                // Upload / QuizFinished → double-tap to exit
                if (!window.__backExitFlag) {
                    window.__backExitFlag = true;
                    try { AndroidBackHandler.showToast('再按一次以退出'); } catch(e) {}
                    setTimeout(function() { window.__backExitFlag = false; }, 3000);
                } else {
                    try { AndroidBackHandler.finishActivity(); } catch(e) {}
                }
            };
            "#;
            let _ = document::eval(js).await;
        });
    });

    let screen = app.screen.read().clone();
    let toast = app.toast.read().clone();
    let update_info = app.update_info.read().clone();
    let download_progress = app.download_progress.read().clone();

    rsx! {
        style { "{css::STYLES}" }
        script {
            dangerous_inner_html: ANTI_FOUC_SCRIPT,
        }

        div { class: "app-shell",
            div { class: "app-content",
                match screen {
                    Screen::Exam => rsx! { ExamScreen {} },
                    Screen::Quiz => rsx! { QuizScreen {} },
                    Screen::QuizFinished => rsx! { QuizFinished {} },
                    Screen::Library => rsx! { LibraryScreen {} },
                    Screen::DeckDetail { .. } => rsx! { DeckDetailScreen {} },
                    Screen::Settings => rsx! { SettingsScreen {} },
                    Screen::Import => rsx! { ImportScreen {} },
                }
            }
            match screen {
                Screen::Exam | Screen::Library | Screen::Settings | Screen::Import => rsx! { NavBar {} },
                _ => rsx! {},
            }
        }

        div {
            class: if toast.is_some() { "toast show" } else { "toast" },
            role: "alert",
            aria_live: "assertive",
            "{toast.as_ref().map(|t| t.text.as_str()).unwrap_or_default()}"
        }

        // update prompt dialog
        {update_info.as_ref().map(|info| {
            let tag = info.tag.strip_prefix('v').unwrap_or(&info.tag).to_string();
            let url = info.url.clone();
            let size = info.size;
            let onclick_update = move |_| {
                app.update_info.set(None);
                app.download_progress.set(Some(0.0));

                let js = format!(
                    "AndroidAppUpdater.downloadAndInstall('{}', {})",
                    url.replace('\'', "\\'"),
                    size,
                );
                spawn(async move { let _ = document::eval(&js).await; });

                let mut app_poll = app;
                spawn(async move {
                    loop {
                        sleep_ms(300).await;
                        let mut eval = document::eval(
                            "dioxus.send(String(AndroidAppUpdater.getProgress()))",
                        );
                        match eval.recv::<String>().await {
                            Ok(s) => {
                                if let Ok(pct) = s.parse::<f64>() {
                                    if pct < 0.0 || pct >= 1.0 { break; }
                                    app_poll.download_progress.set(Some(pct));
                                    continue;
                                }
                            }
                            _ => {}
                        }
                        break;
                    }
                    app_poll.download_progress.set(None);
                });
            };
            let onclick_later = move |_| app.update_info.set(None);
            let skip_tag = info.tag.clone();
            let onclick_skip = move |_| {
                app.update_info.set(None);
                let tag = skip_tag.clone();
                spawn(async move {
                    // flat JSON array in localStorage, no dedup needed at read
                    let js = format!(
                        r#"try{{var s=JSON.parse(localStorage.getItem('skipped_versions')||'[]');if(!s.includes('{t}')){{s.push('{t}');localStorage.setItem('skipped_versions',JSON.stringify(s));}}}}catch(e){{}}"#,
                        t = tag.replace('\'', "\\'"),
                    );
                    let _ = document::eval(&js).await;
                });
            };
            rsx! {
                ModalDialog {
                    visible: true,
                    title: "發現新版本",
                    div { class: "update-body", "v{tag} 已發布，是否下載更新？" }
                    div { class: "update-actions",
                        button { class: "update-btn secondary", onclick: onclick_skip, "略過" }
                        button { class: "update-btn secondary", onclick: onclick_later, "稍後" }
                        button { class: "update-btn primary", onclick: onclick_update, "更新" }
                    }
                }
            }
        })}

        // download progress dialog
        {download_progress.as_ref().map(|&pct| {
            let display = (pct * 100.0) as u32;
            rsx! {
                div { class: "update-overlay",
                    div { class: "update-dialog",
                        div { class: "update-title", "正在下載更新…" }
                        div { class: "update-body dl-progress-body",
                            span { class: "material-symbols-outlined update-dl-icon", "download" }
                            " {display}%"
                        }
                        div { class: "dl-track",
                            div { class: "dl-fill", style: "width: {display}%" }
                        }
                    }
                }
            }
        })}

        // reset confirmation dialog
        ModalDialog {
            visible: *app.show_reset_confirm.read(),
            title: "還原設定",
            div { class: "update-body", "確定要還原所有設定為預設值嗎？" }
            div { class: "update-actions",
                button {
                    class: "update-btn secondary",
                    onclick: move |_| app.show_reset_confirm.set(false),
                    "取消"
                }
                button {
                    class: "update-btn primary",
                    onclick: move |_| {
                        app.fsrs_config.set(FsrsConfig::default());
                        app.theme_mode.set(ThemeMode::System);
                        app.infinite_mode.set(true);
                        app.auto_advance_ms.set(DEFAULT_AUTO_ADVANCE_MS);
                        app.update_check_enabled.set(true);
                        push_toast(app, "已還原預設值");
                        app.show_reset_confirm.set(false);
                    },
                    "確定"
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct StoredPrefs {
    theme: String,
    resolved_dark: bool,
    urls: Vec<String>,
    infinite_mode: Option<bool>,
    show_finished_screen: Option<bool>,
    auto_advance_ms: Option<i64>,
    fsrs_config: Option<String>,
    update_check_enabled: Option<bool>,
}

async fn load_prefs(mut app: AppSignals) {
    let mut eval = document::eval(
        r#"
        try {
            let theme = localStorage.getItem('theme') || '';
            if (!theme) theme = 'system';
            let resolved = theme;
            if (theme === 'system') {
                resolved = (window.AndroidSystemTheme && AndroidSystemTheme.isSystemDark()) ? 'dark' : 'light';
            }
            document.documentElement.setAttribute('data-theme', resolved);
            const urls = JSON.parse(localStorage.getItem('recent_urls') || '[]');
            const infinite_mode = localStorage.getItem('infinite_mode') !== 'false';
            const auto_advance_ms = parseInt(localStorage.getItem('auto_advance_ms'), 10) || null;
            const fsrs_config = localStorage.getItem('fsrs_config') || '';
            const update_check_enabled = localStorage.getItem('update_check_enabled') !== 'false';
            dioxus.send(JSON.stringify({ theme, resolved_dark: resolved === 'dark', urls: Array.isArray(urls) ? urls : [], infinite_mode, auto_advance_ms, fsrs_config, update_check_enabled }));
        } catch (_) {
            document.documentElement.setAttribute('data-theme', 'light');
            dioxus.send(JSON.stringify({ theme: 'system', resolved_dark: false, urls: [], infinite_mode: true, auto_advance_ms: null, fsrs_config: '', update_check_enabled: true }));
        }
        "#,
    );

    if let Ok(payload) = eval.recv::<String>().await {
        if let Ok(prefs) = serde_json::from_str::<StoredPrefs>(&payload) {
            app.theme_mode.set(match prefs.theme.as_str() {
                "dark" => ThemeMode::Dark,
                "light" => ThemeMode::Light,
                _ => ThemeMode::System,
            });
            app.is_dark.set(prefs.resolved_dark);
            app.infinite_mode.set(prefs.infinite_mode.unwrap_or(true));
            app.show_finished_screen.set(prefs.show_finished_screen.unwrap_or(true));
            if let Some(v) = prefs.auto_advance_ms {
                app.auto_advance_ms.set(v);
            }
            app.recent_urls
                .set(clean_recent_urls(prefs.urls, MAX_RECENT_URLS));
            if let Some(json) = prefs.fsrs_config {
                if let Ok(cfg) = serde_json::from_str::<FsrsConfig>(&json) {
                    app.fsrs_config.set(cfg);
                }
            }
            if let Some(v) = prefs.update_check_enabled {
                app.update_check_enabled.set(v);
            }
        } else {
            log!("[Prefs::Load] failed to parse prefs payload");
        }
    } else {
        log!("[Prefs::Load] failed to receive eval result");
    }

    app.prefs_loaded.set(true);
}

async fn persist_theme(mode: ThemeMode, is_dark: bool) {
    let data_theme = if is_dark { "dark" } else { "light" };
    let mode_str = match mode {
        ThemeMode::System => "system",
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    };
    let mode_js = serde_json::to_string(mode_str).unwrap_or_else(|_| "\"system\"".to_string());

    let data_js = serde_json::to_string(data_theme).unwrap_or_else(|_| "\"light\"".to_string());
    let script = format!(
        r#"
        try {{
            document.documentElement.setAttribute('data-theme', {data_js});
            localStorage.setItem('theme', {mode_js});
        }} catch (_) {{
            document.documentElement.setAttribute('data-theme', {data_js});
        }}
        "#
    );

    if let Err(e) = document::eval(&script).await {
        log!("[Prefs::Theme] eval failed: {e}");
    }
}

async fn persist_fsrs_config(cfg: FsrsConfig) {
    let Ok(json) = serde_json::to_string(&cfg) else {
        log!("[Prefs::FsrsConfig] serialize failed");
        return;
    };
    let js = serde_json::to_string(&json).unwrap_or_else(|_| "\"\"".to_string());
    let script = format!(
        r#"try {{ localStorage.setItem('fsrs_config', {js}); }} catch (_) {{}}"#
    );
    if let Err(e) = document::eval(&script).await {
        log!("[Prefs::FsrsConfig] save failed: {e}");
    }
}

async fn persist_infinite_mode(infinite: bool) {
    let val = serde_json::to_string(&infinite).unwrap_or_else(|_| "false".to_string());
    let script = format!(
        r#"try {{ localStorage.setItem('infinite_mode', {val}); }} catch (_) {{}}"#
    );
    if let Err(e) = document::eval(&script).await {
        log!("[Prefs::InfiniteMode] save failed: {e}");
    }
}

async fn persist_show_finished_screen(enabled: bool) {
    let val = serde_json::to_string(&enabled).unwrap_or_else(|_| "true".to_string());
    let script = format!(
        r#"try {{ localStorage.setItem('show_finished_screen', {val}); }} catch (_) {{}}"#
    );
    if let Err(e) = document::eval(&script).await {
        log!("[Prefs::ShowFinished] save failed: {e}");
    }
}

async fn persist_auto_advance_ms(ms: i64) {
    let js = format!(
        r#"try {{ localStorage.setItem('auto_advance_ms', '{ms}'); }} catch (_) {{}}"#
    );
    if let Err(e) = document::eval(&js).await {
        log!("[Prefs::AutoAdvanceMs] save failed: {e}");
    }
}

async fn persist_update_check_enabled(enabled: bool) {
    let js = if enabled {
        r#"try { localStorage.removeItem('update_check_enabled'); } catch(_) {}"#.to_string()
    } else {
        r#"try { localStorage.setItem('update_check_enabled', 'false'); } catch(_) {}"#.to_string()
    };
    if let Err(e) = document::eval(&js).await {
        log!("[Prefs::UpdateCheck] save failed: {e}");
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExamHistoryItem {
    decks: String,
    words: usize,
    correct: usize,
    total: usize,
    date: String,
}

#[component]
fn ExamScreen() -> Element {
    let mut app = use_context::<AppSignals>();

    let mut items = use_signal(Vec::<Deck>::new);
    let mut current_folder = use_signal::<Option<i64>>(|| None);
    let mut breadcrumb = use_signal(Vec::<Deck>::new);
    let mut expanded = use_signal(HashSet::<i64>::new);
    let mut deck_words = use_signal(HashMap::<i64, Vec<(i64, Word)>>::new);
    let mut selected = use_signal(HashSet::<i64>::new);
    let mut search = use_signal(String::new);
    let mut search_mode = use_signal(|| false);
    let mut show_hist = use_signal(|| false);
    let mut history = use_signal(Vec::<ExamHistoryItem>::new);
    let mut deck_colors = use_signal(HashMap::<String, String>::new);
    let mut show_restart_dialog = use_signal(|| false);
    let mut restart_history_item = use_signal::<Option<ExamHistoryItem>>(|| None);

    // Load items at current folder level
    use_effect(move || {
        let _ = *current_folder.read();
        let db = app.db.cloned();
        spawn(async move {
            if let Some(db) = db {
                let fid = current_folder.cloned();
                if let Ok(list) = db.list_by_parent(fid) {
                    items.set(list);
                }
                if let Ok(path) = db.get_folder_path(fid) {
                    breadcrumb.set(path);
                }
                if let Ok(map) = db.all_deck_name_colors() {
                    deck_colors.set(map);
                }
            }
        });
    });

    // Load history from localStorage
    use_effect(move || {
        let _ = *show_hist.read();
        spawn(async move {
            let js = r#"try { dioxus.send(localStorage.getItem('exam_history') || '[]'); } catch(_) { dioxus.send('[]'); }"#;
            let mut eval = document::eval(js);
            if let Ok(json) = eval.recv::<String>().await {
                if let Ok(items) = serde_json::from_str::<Vec<ExamHistoryItem>>(&json) {
                    history.set(items);
                }
            }
        });
    });

    let mut navigate = move |fid: Option<i64>| {
        current_folder.set(fid);
        search.set(String::new());
    };

    let select_all = move |_| {
        let words_map = deck_words.cloned();
        let mut s = HashSet::new();
        for (_, words) in &words_map {
            for (id, _) in words { s.insert(*id); }
        }
        selected.set(s);
    };

    let clear_all = move |_| selected.set(HashSet::new());

    let mut toggle_expand = move |deck_id: i64| {
        let mut e = expanded.cloned();
        if e.contains(&deck_id) { e.remove(&deck_id); } else { e.insert(deck_id); }
        expanded.set(e.clone());

        let dw = deck_words.cloned();
        if !dw.contains_key(&deck_id) {
            let db = app.db.cloned();
            spawn(async move {
                if let Some(db) = db {
                    if let Ok(words) = db.list_words_by_deck(deck_id) {
                        let mut m = deck_words.cloned();
                        m.insert(deck_id, words);
                        deck_words.set(m);
                    }
                }
            });
        }
    };

    let mut toggle_deck = move |deck_id: i64| {
        let dw = deck_words.cloned();
        let loaded = dw.contains_key(&deck_id);
        if !loaded {
            let db = app.db.cloned();
            spawn(async move {
                if let Some(db) = db {
                    if let Ok(words) = db.list_words_by_deck(deck_id) {
                        let mut m = deck_words.cloned();
                        let ids: Vec<i64> = words.iter().map(|(id, _)| *id).collect();
                        m.insert(deck_id, words);
                        deck_words.set(m);
                        let mut s = selected.cloned();
                        let all_selected = ids.iter().all(|id| s.contains(id));
                        if all_selected {
                            for id in &ids { s.remove(id); }
                        } else {
                            for id in &ids { s.insert(*id); }
                        }
                        selected.set(s);
                    }
                }
            });
        } else if let Some(words) = dw.get(&deck_id) {
            let ids: Vec<i64> = words.iter().map(|(id, _)| *id).collect();
            let mut s = selected.cloned();
            if ids.iter().all(|id| s.contains(id)) {
                for id in &ids { s.remove(id); }
            } else {
                for id in &ids { s.insert(*id); }
            }
            selected.set(s);
        }
    };

    let mut toggle_word = move |word_id: i64| {
        let mut s = selected.cloned();
        if s.contains(&word_id) { s.remove(&word_id); } else { s.insert(word_id); }
        selected.set(s);
    };

    let start_exam = move |_| {
        let dw = deck_words.cloned();
        let sel = selected.cloned();
        let all_items = items.cloned();
        let mut words: Vec<Word> = Vec::new();
        let mut deck_names: Vec<String> = Vec::new();
        for (did, wlist) in &dw {
            let mut added = false;
            for (wid, w) in wlist {
                if sel.contains(wid) {
                    words.push(w.clone());
                    if !added { added = true; }
                }
            }
            if added {
                if let Some(d) = all_items.iter().find(|d| d.id == *did) {
                    deck_names.push(d.name.clone());
                } else if let Some(ref db) = *app.db.read() {
                    if let Ok(d) = db.get_deck(*did) {
                        deck_names.push(d.name);
                    }
                }
            }
        }
        if words.is_empty() { return; }
        let wc = words.len();
        let mut qs = QuizState::new(words, *app.infinite_mode.read(), app.fsrs_config.cloned());
        if !qs.gen_question() { return; }
        app.quiz.set(Some(qs));
        let name_str = deck_names.join(" + ");
        app.exam_pending_name.set(Some(ExamPendingName { names: name_str, word_count: wc }));
        app.screen.set(Screen::Quiz);
    };

    let q = search.read().clone();
    let display_items: Vec<DisplayItem> = if q.trim().is_empty() {
        items.cloned().into_iter().map(DisplayItem::Deck).collect()
    } else {
        let mut results: Vec<DisplayItem> = Vec::new();
        if let Some(ref db) = *app.db.read() {
            if let Ok(decks) = db.search_decks(&q) {
                let mut groups: HashMap<Option<i64>, Vec<Deck>> = HashMap::new();
                for d in decks {
                    if !d.is_folder {
                        groups.entry(d.parent_id).or_default().push(d);
                    }
                }
                if !groups.is_empty() {
                    let folder_map = db.get_folder_map(&[]).ok().unwrap_or_default();
                    let mut group_entries: Vec<(Option<i64>, Vec<Deck>)> = groups.drain().collect();
                    for (_, list) in &mut group_entries {
                        list.sort_by(|a, b| a.name.cmp(&b.name));
                    }
                    group_entries.sort_by(|(pa, _), (pb, _)| pa.cmp(pb));
                    for (pid, decks) in group_entries {
                        let name = match pid {
                            None => "根目錄".to_string(),
                            Some(pid) => {
                                let mut path = Vec::new();
                                let mut cur = Some(pid);
                                while let Some(fid) = cur {
                                    if let Some((n, p)) = folder_map.get(&fid) {
                                        path.push(n.clone());
                                        cur = *p;
                                    } else {
                                        path.push("未知資料夾".to_string());
                                        break;
                                    }
                                }
                                path.reverse();
                                path.join(" / ")
                            }
                        };
                        results.push(DisplayItem::Header(name));
                        for d in decks {
                            results.push(DisplayItem::Deck(d));
                        }
                    }
                }
            }
        }
        results
    };

    let selected_count = selected.read().len();
    let dw_map = deck_words.cloned();
    let sel_set = selected.cloned();
    let hist_list = history.cloned();
    let hist_visible = *show_hist.read();
    let bc = breadcrumb.cloned();

    rsx! {
        div { class: "exam-screen",
            div { class: "exam-topbar",
                span { class: format!("exam-title{}", if *search_mode.read() { " search-mode" } else { "" }), "測驗" }
                div { class: format!("exam-search-bar{}", if *search_mode.read() { " search-mode" } else { "" }),
                    div { class: "exam-search-wrap",
                        span { class: "material-symbols-outlined exam-search-icon", "search" }
                        input {
                            class: "exam-search-input",
                            r#type: "text",
                            placeholder: "搜尋牌組...",
                            value: "{search}",
                            oninput: move |e| search.set(e.value()),
                        }
                    }
                    button {
                        class: "search-close",
                        onclick: move |_| {
                            search_mode.set(false);
                            search.set(String::new());
                        },
                        span { class: "material-symbols-outlined", "close" }
                    }
                }
                if !*search_mode.read() {
                    button {
                        class: "exam-topbar-btn",
                        onclick: move |_| search_mode.set(true),
                        span { class: "material-symbols-outlined", "search" }
                    }
                }
                div { style: "position: relative; display: flex;",
                    button {
                        class: "exam-topbar-btn",
                        onclick: move |_| {
                            let h = *show_hist.read();
                            show_hist.set(!h);
                        },
                        span { class: "material-symbols-outlined", "schedule" }
                    }
                    if hist_visible {
                        div { class: "history-overlay open",
                            onclick: move |_| show_hist.set(false),
                        }
                        div { class: "history-panel open",
                            div { class: "history-panel-title", "最近測驗紀錄" }
                            if hist_list.is_empty() {
                                div { class: "history-empty", "尚無測驗紀錄" }
                            } else {
                                {hist_list.into_iter().map(|h| {
                                    let pct = if h.total > 0 { (h.correct as f64 / h.total as f64 * 100.0).round() as u32 } else { 0 };
                                    let score_color = if pct >= 85 { "#43a047" } else if pct >= 60 { "#FFB300" } else { "#e53935" };
                                    let first_deck = h.decks.split(" + ").next().unwrap_or("");
                                    let dot_color = deck_colors.read().get(first_deck).cloned().unwrap_or_else(|| score_color.to_string());
                                    let item = h.clone();
                                    rsx! {
                                        button {
                                            class: "history-item",
                                            onclick: move |_| {
                                                restart_history_item.set(Some(item.clone()));
                                                show_restart_dialog.set(true);
                                                show_hist.set(false);
                                            },
                                            span { style: "width: 10px; height: 10px; border-radius: 50%; background: {dot_color}; flex-shrink: 0;" }
                                            div { class: "history-item-body",
                                                div { class: "history-item-name", "{h.decks}" }
                                                div { class: "history-item-meta", "{h.date} · {h.words} 詞" }
                                            }
                                            span { class: "history-item-score", style: "color: {score_color}", "{pct}%" }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }
            }
            // Breadcrumb
            div { class: "exam-breadcrumb",
                button {
                    class: "bc-btn",
                    onclick: move |_| navigate(None),
                    span { class: "material-symbols-outlined", "folder" }
                }
                {bc.into_iter().map(|f| {
                    let fid = f.id;
                    let fname = f.name.clone();
                    rsx! {
                        span { class: "bc-sep", ">" }
                        button {
                            class: "bc-btn",
                            onclick: move |_| navigate(Some(fid)),
                            "{fname}"
                        }
                    }
                })}
            }
            div { class: "selection-bar",
                span { class: "selection-chip",
                    span { class: "material-symbols-outlined", "checklist" }
                    span { "已選 {selected_count} 個單字" }
                }
                div { class: "selection-actions",
                    button { onclick: select_all, "全選" }
                    button { onclick: clear_all, "清除" }
                }
            }
            div { class: "deck-list",
                if display_items.is_empty() {
                    div { class: "empty-state",
                        span { class: "material-symbols-outlined", "search_off" }
                        span { "沒有符合的牌組" }
                    }
                } else {
                    {display_items.into_iter().map(|item| {
                        match item {
                            DisplayItem::Header(name) => {
                                rsx! {
                                    div { class: "search-group-header",
                                        span { class: "material-symbols-outlined search-group-icon", "folder" }
                                        span { class: "search-group-title", "{name}" }
                                    }
                                }
                            }
                            DisplayItem::Deck(item) => {
                                if item.is_folder {
                                    let fid = item.id;
                                    let fname = item.name.clone();
                                    rsx! {
                                        div {
                                            key: "f-{fid}",
                                            class: "folder-item",
                                            onclick: move |_| navigate(Some(fid)),
                                            span { class: "material-symbols-outlined folder-icon", "folder" }
                                            div { class: "deck-info",
                                                div { class: "deck-name", "{fname}" }
                                            }
                                        }
                                    }
                                } else {
                                    let did = item.id;
                                    let dname = item.name.clone();
                                    let dcolor = item.color.clone();
                                    let wc = item.word_count;
                                    let is_expanded = expanded.read().contains(&did);
                                    let loaded_words = dw_map.get(&did).cloned().unwrap_or_default();
                                    let ids: Vec<i64> = loaded_words.iter().map(|(id, _)| *id).collect();
                                    let all_sel = !ids.is_empty() && ids.iter().all(|id| sel_set.contains(id));
                                    let any_sel = !ids.is_empty() && ids.iter().any(|id| sel_set.contains(id));
                                    let cb_class = if all_sel { "deck-checkbox checked" } else if any_sel { "deck-checkbox indet" } else { "deck-checkbox" };

                                    rsx! {
                                        div { key: "d-{did}", class: "deck-card",
                                            div {
                                                class: "deck-card-header",
                                                onclick: move |_| toggle_deck(did),
                                                div {
                                                    class: "{cb_class}",
                                                    onclick: move |e| { e.stop_propagation(); toggle_deck(did); },
                                                    if all_sel {
                                                        span { class: "material-symbols-outlined", "check" }
                                                    }
                                                }
                                                span { class: "deck-color-dot", style: "background: {dcolor}" }
                                                div { class: "deck-info",
                                                    div { class: "deck-name", "{dname}" }
                                                    div { class: "deck-meta",
                                                        span { "{wc} 詞" }
                                                        if any_sel && !all_sel {
                                                            span { "· 部分" }
                                                        }
                                                    }
                                                }
                                                button {
                                                    class: format!("deck-expand{}", if is_expanded { " open" } else { "" }),
                                                    onclick: move |e| { e.stop_propagation(); toggle_expand(did); },
                                                    span { class: "material-symbols-outlined", "expand_more" }
                                                }
                                            }
                                            div { class: format!("word-sublist{}", if is_expanded { " open" } else { "" }),
                                                {loaded_words.into_iter().map(|(wid, w)| {
                                                    let w_sel = sel_set.contains(&wid);
                                                    rsx! {
                                                        div {
                                                            key: "w-{wid}",
                                                            class: "word-item",
                                                            onclick: move |_| toggle_word(wid),
                                                            div {
                                                                class: format!("word-checkbox{}", if w_sel { " checked" } else { "" }),
                                                                if w_sel {
                                                                    span { class: "material-symbols-outlined", "check" }
                                                                }
                                                            }
                                                            span { class: "word-front", "{w.front}" }
                                                            span { class: "word-back", "{w.back}" }
                                                        }
                                                    }
                                                })}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    })}
                }
            }
            button {
                class: format!("exam-fab{}", if selected_count > 0 { " visible" } else { "" }),
                onclick: start_exam,
                span { class: "material-symbols-outlined", "play_arrow" }
                span { "開始測驗" }
                if selected_count > 0 {
                    span { class: "fab-badge", "{selected_count}" }
                }
            }
        }
        ModalDialog {
            visible: *show_restart_dialog.read(),
            title: "重新測驗",
            div { class: "update-body",
                {restart_history_item.read().as_ref().map(|h| {
                    rsx! { "重新開始「{h.decks}」的測驗？" }
                })}
            }
            div { class: "update-actions",
                button {
                    class: "update-btn secondary",
                    onclick: move |_| {
                        show_restart_dialog.set(false);
                        restart_history_item.set(None);
                    },
                    "取消"
                }
                button {
                    class: "update-btn primary",
                    onclick: move |_| {
                        let h = restart_history_item.cloned();
                        show_restart_dialog.set(false);
                        restart_history_item.set(None);
                        if let Some(h) = h {
                            spawn(async move {
                                let db = app.db.cloned();
                                let Some(db) = db else { return };
                                let deck_names: Vec<&str> = h.decks.split(" + ").collect();
                                let mut words: Vec<Word> = Vec::new();
                                for name in deck_names {
                                    if let Ok(decks) = db.search_decks(name) {
                                        for d in decks {
                                            if d.is_folder || d.name != name { continue; }
                                            if let Ok(wlist) = db.list_words_by_deck(d.id) {
                                                for (_, w) in wlist {
                                                    words.push(w);
                                                }
                                            }
                                        }
                                    }
                                }
                                if words.is_empty() { push_toast(app, "找不到對應牌組\n可能已被刪除或重新命名"); return; }
                                let wc = words.len();
                                let mut qs = QuizState::new(words, *app.infinite_mode.read(), app.fsrs_config.cloned());
                                if !qs.gen_question() { return; }
                                app.quiz.set(Some(qs));
                                app.exam_pending_name.set(Some(ExamPendingName { names: h.decks, word_count: wc }));
                                app.screen.set(Screen::Quiz);
                            });
                        }
                    },
                    "確定"
                }
            }
        }
    }
}

type FetchResult<T> = Result<T, String>;

async fn save_exam_history(app: &AppSignals, quiz: &QuizState) {
    let Some(pending) = app.exam_pending_name.read().clone() else { return };
    let correct = quiz.history.iter().filter(|h| h.answered && !h.skipped && h.selected_idx == Some(h.correct_opt)).count();
    let total = quiz.history.iter().filter(|h| h.answered && !h.skipped).count();

    let mut eval_date = document::eval(
        r#"var d=new Date();var mm=(d.getMonth()+1);var dd=d.getDate();var h=d.getHours();var m=d.getMinutes();dioxus.send((mm<10?'0':'')+mm+'/'+(dd<10?'0':'')+dd+' '+(h<10?'0':'')+h+':'+(m<10?'0':'')+m)"#,
    );
    let date_str = eval_date.recv::<String>().await.unwrap_or_default();
    let item = ExamHistoryItem {
        decks: pending.names,
        words: pending.word_count,
        correct,
        total,
        date: date_str,
    };
    let mut eval_load = document::eval(
        r#"try { dioxus.send(localStorage.getItem('exam_history') || '[]'); } catch(_) { dioxus.send('[]'); }"#,
    );
    let existing_json = eval_load.recv::<String>().await.unwrap_or_else(|_| "[]".into());
    let mut vec: Vec<ExamHistoryItem> = serde_json::from_str(&existing_json).unwrap_or_default();
    vec.insert(0, item);
    if vec.len() > 10 { vec.truncate(10); }
    if let Ok(new_json) = serde_json::to_string(&vec) {
        if let Ok(js_literal) = serde_json::to_string(&new_json) {
            let js_save = format!(r#"try{{localStorage.setItem('exam_history', {});}}catch(e){{}}"#, js_literal);
            let _ = document::eval(&js_save).await;
        }
    }
}

pub(crate) async fn fetch_html_via_webview(url: &str) -> FetchResult<String> {
    let url_js = serde_json::to_string(url).unwrap_or_else(|_| "\"\"".to_string());
    let js = format!(
        r#"
        if (typeof AndroidQuizletFetcher !== 'undefined') {{
            try {{
                window.__quizletFetchDone = false;
                window.__quizletFetchComplete = function(html) {{
                    if (window.__quizletFetchDone) return;
                    window.__quizletFetchDone = true;
                    clearTimeout(window.__quizletFetchTimeout);
                    dioxus.send(html || '');
                }};
                window.__quizletFetchTimeout = setTimeout(function() {{
                    if (!window.__quizletFetchDone) {{
                        window.__quizletFetchDone = true;
                        dioxus.send('');
                    }}
                }}, 20000);
                AndroidQuizletFetcher.fetchQuizlet({url_js});
            }} catch(e) {{
                dioxus.send('');
            }}
        }} else {{
            try {{
                let resp = await fetch({url_js}, {{
                    credentials: 'include',
                    headers: {{ 'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8' }},
                }});
                if (!resp.ok) {{ dioxus.send(''); return; }}
                let text = await resp.text();
                dioxus.send(text);
            }} catch(e) {{
                dioxus.send('');
            }}
        }}
        "#
    );
    let mut eval = document::eval(&js);
    match tokio::time::timeout(Duration::from_secs(25), eval.recv::<String>()).await {
        Ok(Ok(s)) if !s.is_empty() => Ok(s),
        Ok(Ok(_)) => Err("WebView fetch returned empty".to_string()),
        Ok(Err(e)) => Err(format!("WebView eval failed: {e}")),
        Err(_) => Err("WebView fetch timed out after 25s".to_string()),
    }
}

#[component]
fn QuizScreen() -> Element {
    let mut app = use_context::<AppSignals>();
    let mut show_pause = use_signal(|| false);
    let mut auto_armed = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            sleep_ms(50).await;
            let _ = document::eval("document.querySelector('.quiz-container')?.focus();").await;
        });
    });

    // auto-advance after answering the last question
    use_effect(move || {
        let auto_ms = *app.auto_advance_ms.read();
        let (is_answered, is_last, current_idx) = {
            let qs = app.quiz.read();
            let qs = match qs.as_ref() {
                Some(qs) => qs,
                None => return,
            };
            let q = match qs.current_question() {
                Some(q) => q,
                None => return,
            };
            (q.answered, qs.current + 1 == qs.history.len(), qs.current)
        };

        if is_answered && is_last {
            let has_more = app.quiz.read().as_ref().is_some_and(|qs| qs.has_more());
            if !has_more {
                app.screen.set(Screen::QuizFinished);
                return;
            }
            if auto_ms < 0 { return; }
            if !auto_armed() {
                auto_armed.set(true);
                spawn(async move {
                    sleep_ms(auto_ms as u64).await;
                    let mut guard = app.quiz.write();
                    if let Some(qs) = guard.as_mut() {
                        if qs.current == current_idx {
                            qs.next();
                        }
                    }
                    auto_armed.set(false);
                });
            }
        } else {
            auto_armed.set(false);
        }
    });

    let (correct_count, wrong_count, review_count) = {
        let qs = app.quiz.read();
        match qs.as_ref() {
            Some(qs) => {
                let ok = qs.history.iter().filter(|h| h.answered && !h.skipped && h.selected_idx == Some(h.correct_opt)).count();
                let ko = qs.history.iter().filter(|h| h.answered && !h.skipped && h.selected_idx != Some(h.correct_opt)).count();
                (ok, ko, ko)
            }
            None => (0, 0, 0),
        }
    };

    rsx! {
        div { class: "quiz-screen",
            button {
                class: "top-icon-btn",
                style: "right: 20px;",
                onclick: move |_| show_pause.set(true),
                span { class: "material-symbols-outlined", "pause" }
            }
            section {
                class: "quiz-container",
                tabindex: "0",
                aria_label: "單字測驗區域",
                onkeydown: move |e: KeyboardEvent| {
                let mut guard = app.quiz.write();
                let qs = match guard.as_mut() {
                    Some(qs) => qs,
                    None => return,
                };
                if e.key() == Key::Escape {
                    show_pause.set(true);
                    return;
                }
                let (answered, opt_len) = match qs.current_question() {
                    Some(q) => (q.answered, q.options.len()),
                    None => return,
                };
                match e.key() {
                    Key::Character(ref s) if !answered => {
                        match s.as_str() {
                            "1" | "!" if opt_len > 0 => qs.answer(0),
                            "2" | "@" if opt_len > 1 => qs.answer(1),
                            "3" | "#" if opt_len > 2 => qs.answer(2),
                            "4" | "$" if opt_len > 3 => qs.answer(3),
                            "0" | ")" => qs.skip(),
                            _ => {}
                        }
                    }
                    Key::ArrowRight | Key::Enter if answered => {
                        qs.next();
                        if !qs.has_more() { app.screen.set(Screen::QuizFinished); }
                    }
                    Key::ArrowLeft if qs.current > 0 => { qs.prev(); }
                    _ => {}
                }
            },
            div { class: "back-bar",
                span { class: "q-correct", "{correct_count}" }
                span { class: "q-plus", "+" }
                span { class: "q-wrong", "{wrong_count}" }
                if review_count > 0 && app.fsrs_config.read().review_wrong {
                    span { class: "badge badge-review", span { class: "material-symbols-outlined", "sync" }, " 待複習 {review_count}" }
                }
            }
            QuestionDisplay {}
            OptionsList {}
            ControlButtons {}
        }
        if app.fsrs_config.read().enabled && app.fsrs_config.read().manual_rating {
            FsrsRatingBar {}
        }
        }
        if *show_pause.read() {
            div { class: "pause-overlay",
                div { class: "pause-dialog",
                    div { class: "pause-title", "暫停選單" }
                    div { class: "pause-btn-row",
                        button {
                            class: "pause-icon-box",
                            onclick: move |_| {
                                let qs = app.quiz.cloned();
                                spawn(async move {
                                    if let Some(ref qs) = qs {
                                        save_exam_history(&app, qs).await;
                                    }
                                    app.exam_pending_name.set(None);
                                    app.quiz.set(None);
                                    app.screen.set(Screen::Exam);
                                });
                            },
                            span { class: "material-symbols-outlined", "home" }
                            span { class: "pause-btn-label", "首頁" }
                        }
                        button {
                            class: "pause-icon-box",
                            onclick: move |_| show_pause.set(false),
                            span { class: "material-symbols-outlined", "play_arrow" }
                            span { class: "pause-btn-label", "繼續" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn QuestionDisplay() -> Element {
    let app = use_context::<AppSignals>();

    let info = {
        let qs = app.quiz.read();
        let Some(qs) = qs.as_ref() else {
            return rsx! { div {} };
        };
        let Some(q) = qs.current_question() else {
            return rsx! { div {} };
        };
        let word = &qs.words[q.target_idx];
        (word.front.clone(), word.back.clone(), word.pos.clone(), word.pron.clone(), word.example.clone(), word.synonym.clone(), word.antonym.clone(), q.answered, q.ask_front)
    };
    let (front, back, pos, pron, example, synonym, antonym, answered, ask_front) = info;

    if answered {
        rsx! {
            h2 { id: "question-word",
                span { class: "ans-en", "{front}" }
                span { class: "ans-zh", "{back}" }
            }
            div { class: "word-detail",
                {(!pos.is_empty()).then(|| rsx! {
                    span { class: "word-detail-tag", "{pos}" }
                })}
                {(!pron.is_empty()).then(|| rsx! {
                    span { class: "word-detail-item",
                        span { class: "word-detail-label", "發音" }
                        span { "{pron}" }
                    }
                })}
                {(!example.is_empty()).then(|| rsx! {
                    div { class: "word-detail-item word-detail-example",
                        span { class: "word-detail-label", "例句" }
                        span { "{example}" }
                    }
                })}
                {(!synonym.is_empty()).then(|| rsx! {
                    div { class: "word-detail-item",
                        span { class: "word-detail-label", "同義詞" }
                        span { "{synonym}" }
                    }
                })}
                {(!antonym.is_empty()).then(|| rsx! {
                    div { class: "word-detail-item",
                        span { class: "word-detail-label", "反義詞" }
                        span { "{antonym}" }
                    }
                })}
            }
        }
    } else {
        let text = if ask_front { &front } else { &back };
        rsx! { h2 { id: "question-word", "{text}" } }
    }
}

struct OptData {
    label: String,
    display: String,
    pair: String,
    answered: bool,
    idx: usize,
    correct_opt: usize,
    selected_idx: Option<usize>,
    current: usize,
}

#[component]
fn OptionsList() -> Element {
    let mut app = use_context::<AppSignals>();
    let labels = ["1", "2", "3", "4"];

    let button_data: Vec<OptData> = {
        let qs = app.quiz.read();
        let Some(qs) = qs.as_ref() else {
            return rsx! { div {} };
        };
        let Some(q) = qs.current_question() else {
            return rsx! { div {} };
        };
        q.options
            .iter()
            .enumerate()
            .map(|(idx, &word_idx)| {
                let opt_word = &qs.words[word_idx];
                let (display, pair) = if q.ask_front {
                    (opt_word.back.clone(), opt_word.front.clone())
                } else {
                    (opt_word.front.clone(), opt_word.back.clone())
                };
                OptData {
                    label: labels[idx].to_owned(),
                    display,
                    pair,
                    answered: q.answered,
                    idx,
                    correct_opt: q.correct_opt,
                    selected_idx: q.selected_idx,
                    current: qs.current,
                }
            })
            .collect()
    };

    rsx! {
        div { class: "options-container",
            {button_data.into_iter().map(|data| {
                let onclick = move |_| {
                    let mut guard = app.quiz.write();
                    if let Some(qs) = guard.as_mut() {
                        qs.answer(data.idx);
                    }
                };
                let cls = {
                    let mut base = "option-btn".to_owned();
                    if data.answered {
                        if data.idx == data.correct_opt {
                            base.push_str(" correct");
                        } else if data.selected_idx == Some(data.idx) {
                            base.push_str(" wrong");
                        } else {
                            base.push_str(" dimmed");
                        }
                    }
                    base
                };
                rsx! {
                    button {
                        key: "{data.current}_{data.idx}",
                        class: "{cls}",
                        disabled: data.answered,
                        onclick,
                        span { class: "opt-label", "{data.label}" }
                        div { class: "opt-text",
                            span { class: "opt-main", "{data.display}" }
                            if data.answered {
                                span { class: "opt-pair", "{data.pair}" }
                            }
                        }
                    }
                }
            })}
        }
    }
}

#[component]
fn ControlButtons() -> Element {
    let mut app = use_context::<AppSignals>();

    let (answered, can_prev) = {
        let qs = app.quiz.read();
        let Some(qs) = qs.as_ref() else {
            return rsx! { div {} };
        };
        let Some(q) = qs.current_question() else {
            return rsx! { div {} };
        };
        (q.answered, qs.current > 0)
    };

    rsx! {
        div { class: "controls",
            button {
                class: "ctrl-btn outlined",
                disabled: !can_prev,
                onclick: move |_| {
                    let mut guard = app.quiz.write();
                    if let Some(qs) = guard.as_mut() {
                        qs.prev();
                    }
                },
                span { class: "material-symbols-outlined", "navigate_before" } " 上一題"
            }

            button {
                class: "ctrl-btn tonal",
                disabled: answered,
                onclick: move |_| {
                    let mut guard = app.quiz.write();
                    if let Some(qs) = guard.as_mut() {
                        qs.skip();
                    }
                },
                span { class: "material-symbols-outlined", "skip_next" } " 跳過"
            }

            button {
                class: "ctrl-btn filled",
                disabled: !answered,
                onclick: move |_| {
                    let done = {
                        let mut guard = app.quiz.write();
                        let Some(qs) = guard.as_mut() else { return };
                        qs.next();
                        !qs.has_more()
                    };
                    if done { app.screen.set(Screen::QuizFinished); }
                },
                span { class: "material-symbols-outlined", "navigate_next" } " 下一題"
            }
        }
    }
}

#[component]
fn FsrsSettings() -> Element {
    let mut app = use_context::<AppSignals>();
    let cfg = app.fsrs_config.cloned();

    // Local state for validation
    let hard_err: Signal<String> = use_signal(String::new);
    let good_err: Signal<String> = use_signal(String::new);
    let easy_err: Signal<String> = use_signal(String::new);

    rsx! {
        div {
            class: "settings-item",
            onclick: move |_| {
                let mut c = app.fsrs_config.cloned();
                c.enabled = !c.enabled;
                app.fsrs_config.set(c);
            },
            div { class: "settings-item-icon",
                span { class: "material-symbols-outlined", "psychology" }
            }
            div { class: "settings-item-label",
                div { "FSRS 間隔重複" }
                div { class: "settings-item-sub", "啟用 FSRS-6 演算法安排複習" }
            }
            div {
                class: if cfg.enabled { "settings-switch on" } else { "settings-switch" },
            }
        }
        div {
            class: "settings-item",
            onclick: move |_| {
                let mut c = app.fsrs_config.cloned();
                c.manual_rating = !c.manual_rating;
                app.fsrs_config.set(c);
            },
            div { class: "settings-item-icon",
                span { class: "material-symbols-outlined", "rate_review" }
            }
            div { class: "settings-item-label",
                div { "手動評分" }
                div { class: "settings-item-sub", "作答後顯示評分按鈕，關閉則自動評分" }
            }
            div {
                class: if cfg.manual_rating { "settings-switch on" } else { "settings-switch" },
            }
        }
        div { class: "fsrs-threshold-section",
            div { class: "fsrs-threshold-header", "判定時間設定 (毫秒)" }
            div { class: "fsrs-threshold-grid",
                FsrsThresholdInput {
                    field: "easy",
                    label: "簡單",
                    value: cfg.easy_threshold_ms,
                    err: easy_err,
                }
                FsrsThresholdInput {
                    field: "good",
                    label: "良好",
                    value: cfg.good_threshold_ms,
                    err: good_err,
                }
                FsrsThresholdInput {
                    field: "hard",
                    label: "困難",
                    value: cfg.hard_threshold_ms,
                    err: hard_err,
                }
            }
        }
    }
}

#[component]
fn FsrsThresholdInput(field: String, label: String, value: u64, mut err: Signal<String>) -> Element {
    let mut app = use_context::<AppSignals>();
    let input_id = format!("fsrs-{field}");
    let has_err = !err.read().is_empty();
    let input_cls = format!("fsrs-input{}", if has_err { " error" } else { "" });
    let mut text: Signal<String> = use_signal(|| value.to_string());
    use_effect(move || {
        let _ = value;
        text.set(value.to_string());
    });

    rsx! {
        div { class: "fsrs-field",
            label {
                class: "fsrs-label",
                r#for: "{input_id}",
                "{label}"
            }
            input {
                id: "{input_id}",
                class: "{input_cls}",
                r#type: "number",
                value: "{text.read()}",
                placeholder: "毫秒",
                oninput: move |e| {
                    let v = e.value().trim().to_string();
                    text.set(v.clone());
                    if v.is_empty() || v.parse::<u64>().is_ok_and(|n| n == 0) {
                        err.set("請輸入自然數".to_string());
                        return;
                    }
                    match v.parse::<u64>() {
                        Ok(n) if n > 0 => {
                            let mut c = app.fsrs_config.cloned();
                            match field.as_str() {
                                "easy" if n >= c.good_threshold_ms => {
                                    err.set("值必須小於 良好".to_string());
                                    return;
                                }
                                "easy" => c.easy_threshold_ms = n,
                                "good" if n <= c.easy_threshold_ms => {
                                    err.set("值必須大於 簡單".to_string());
                                    return;
                                }
                                "good" if n >= c.hard_threshold_ms => {
                                    err.set("值必須小於 困難".to_string());
                                    return;
                                }
                                "good" => c.good_threshold_ms = n,
                                "hard" if n <= c.good_threshold_ms => {
                                    err.set("值必須大於 良好".to_string());
                                    return;
                                }
                                "hard" => c.hard_threshold_ms = n,
                                _ => {}
                            }
                            err.set(String::new());
                            app.fsrs_config.set(c);
                        }
                        _ => {
                            err.set("請輸入自然數".to_string());
                        }
                    }
                },
            }
            {has_err.then(|| rsx! {
                div { class: "fsrs-error", "{err.read().clone()}" }
            })}
        }
    }
}

#[component]
fn FsrsRatingBar() -> Element {
    let mut app = use_context::<AppSignals>();

    let (show, current_rating) = {
        let qs = app.quiz.read();
        let Some(qs) = qs.as_ref() else {
            return rsx! { div {} };
        };
        let Some(q) = qs.current_question() else {
            return rsx! { div {} };
        };
        let show = q.answered;
        let r = if show { q.rating() } else { None };
        (show, r)
    };

    if !show {
        return rsx! { div {} };
    }

    let ratings = [
        (FsrsRating::Again, "fsrs-btn-again"),
        (FsrsRating::Hard, "fsrs-btn-hard"),
        (FsrsRating::Good, "fsrs-btn-good"),
        (FsrsRating::Easy, "fsrs-btn-easy"),
    ];

    rsx! {
        div { class: "rating-section",
            div { class: "rating-label", "評分" }
            div { class: "fsrs-rating-row",
                {ratings.into_iter().map(|(r, cls_name)| {
                    let selected = current_rating == Some(r);
                    let cls = format!(
                        "fsrs-rating-btn {} {}",
                        cls_name,
                        if selected { "selected" } else { "" }
                    );
                    rsx! {
                        button {
                            class: "{cls}",
                            onclick: move |_| {
                                let mut guard = app.quiz.write();
                                if let Some(qs) = guard.as_mut() {
                                    qs.set_rating(r);
                                }
                            },
                            if selected {
                                span { class: "material-symbols-outlined", "check" }
                            }
                            span { "{r.label()}" }
                        }
                    }
                })}
            }
        }
    }
}

#[component]
fn QuizFinished() -> Element {
    let mut app = use_context::<AppSignals>();

    use_effect(move || {
        let qs = app.quiz.cloned();
        spawn(async move {
            if let Some(ref qs) = qs {
                save_exam_history(&app, qs).await;
                app.exam_pending_name.set(None);
            }
        });
    });

    let (correct, wrong, show_score) = {
        let qs = app.quiz.read();
        let qs = match qs.as_ref() {
            Some(qs) => qs,
            None => return rsx! { div {} },
        };
        let ok = qs.history.iter().filter(|h| h.answered && !h.skipped && h.selected_idx == Some(h.correct_opt)).count();
        let ko = qs.history.iter().filter(|h| h.answered && !h.skipped && h.selected_idx != Some(h.correct_opt)).count();
        let show = *app.show_finished_screen.read();
        (ok, ko, show)
    };
    let total = correct + wrong;
    let pct = if total > 0 { (correct as f64 / total as f64 * 100.0).round() as u32 } else { 0 };
    let score_color = if pct >= 85 { "#43a047" } else if pct >= 61 { "#FFB300" } else { "#e53935" };
    let r = 80.0;
    let circ = 2.0 * std::f64::consts::PI * r;
    let mut cur_off = use_signal(|| circ);
    use_effect(move || {
        let off = circ * (1.0 - pct as f64 / 100.0);
        spawn(async move { sleep_ms(80).await; cur_off.set(off); });
    });

    rsx! {
        div { class: "finish-screen",
            if show_score {
                svg {
                    width: "200", height: "200", view_box: "0 0 200 200",
                    circle {
                        cx: "100", cy: "100", r: "{r}",
                        fill: "none", stroke: "#e0e0e0", stroke_width: "10",
                    }
                    circle {
                        cx: "100", cy: "100", r: "{r}",
                        fill: "none", stroke: "{score_color}",
                        stroke_width: "10", stroke_linecap: "round",
                        stroke_dasharray: "{circ}",
                        stroke_dashoffset: "{cur_off}",
                        transform: "rotate(-90, 100, 100)",
                        style: "transition: stroke-dashoffset 1s cubic-bezier(0.4, 0, 0.2, 1);",
                    }
                    text {
                        x: "100", y: "100",
                        text_anchor: "middle", dominant_baseline: "central",
                        font_size: "32", font_weight: "700",
                        fill: "{score_color}",
                        "{pct}"
                    }
                }
            } else {
                div { class: "finish-icon",
                    span { class: "material-symbols-outlined", "check_circle" }
                }
            }
            div { class: "finish-title", "測驗完成" }
            if show_score {
                div { class: "finish-score",
                    span { class: "correct", "{correct}" } " 題正確　"
                    span { class: "wrong", "{wrong}" } " 題錯誤"
                }
            }
            button {
                class: "finish-btn filled",
                onclick: move |_| {
                    let old_words = {
                        let qs = app.quiz.read();
                        qs.as_ref().map(|qs| qs.words.clone()).unwrap_or_default()
                    };
                    if old_words.is_empty() { return; }
                    let mut qs = QuizState::new(old_words, *app.infinite_mode.read(), app.fsrs_config.cloned());
                    qs.gen_question();
                    app.quiz.set(Some(qs));
                    app.screen.set(Screen::Quiz);
                },
                "再來一次"
            }
            button {
                class: "finish-btn outlined",
                onclick: move |_| {
                    let qs = app.quiz.cloned();
                    spawn(async move {
                        if let Some(ref qs) = qs {
                            save_exam_history(&app, qs).await;
                        }
                        app.exam_pending_name.set(None);
                        app.quiz.set(None);
                        app.screen.set(Screen::Exam);
                    });
                },
                "返回主頁"
            }
        }
    }
}

#[component]
fn NavBar() -> Element {
    let mut app = use_context::<AppSignals>();
    let current = app.screen.read().clone();

    let items = [
        (Screen::Library, "menu_book", "字庫"),
        (Screen::Exam, "quiz", "考試"),
        (Screen::Import, "file_upload", "匯入"),
        (Screen::Settings, "settings", "設定"),
    ];

    rsx! {
        nav { class: "navbar",
            {items.into_iter().map(|(screen, icon, label)| {
                let active = current == screen;
                let cls = if active { "nav-item active" } else { "nav-item" };
                rsx! {
                    button {
                        key: "{label}",
                        class: "{cls}",
                        onclick: move |_| app.screen.set(screen.clone()),
                        span { class: "nav-icon material-symbols-outlined", "{icon}" }
                        span { class: "nav-label", "{label}" }
                    }
                }
            })}
        }
    }
}

enum DisplayItem {
    Header(String),
    Deck(Deck),
}

enum GroupedItem {
    Header { name: String, count: usize },
    Deck(Deck),
}

const DECK_COLORS: &[&str] = &[
    "#c62828", "#e53935", "#d81b60", "#8e24aa",
    "#5e35b1", "#3949ab", "#1e88e5", "#00acc1",
    "#00897b", "#43a047", "#7cb342", "#c0ca33",
    "#fdd835", "#ffb300", "#fb8c00", "#6d4c41",
];

#[component]
fn LibraryScreen() -> Element {
    let mut app = use_context::<AppSignals>();

    let mut current_folder_id = use_signal(|| None::<i64>);
    let mut breadcrumb = use_signal(Vec::<Deck>::new);
    let mut items = use_signal(Vec::<Deck>::new);

    let mut show_fab_menu = use_signal(|| false);
    let mut show_create_deck = use_signal(|| false);
    let mut show_create_folder = use_signal(|| false);
    let mut sort_by = use_signal(|| "date".to_string());
    let mut sort_asc = use_signal(|| true);
    let mut folder_first = use_signal(|| true);
    let mut show_sort_menu = use_signal(|| false);
    let mut search_mode = use_signal(|| false);
    let mut search_text = use_signal(String::new);
    let mut show_rename = use_signal(|| false);
    let mut show_delete = use_signal(|| false);
    let mut rename_target = use_signal(|| 0i64);
    let mut delete_target = use_signal(|| 0i64);
    let mut is_folder_target = use_signal(|| false);
    let mut create_name = use_signal(String::new);
    let mut rename_name = use_signal(String::new);
    let mut refresh = use_signal(|| 0u64);
    let mut selected_color = use_signal(|| DECK_COLORS[0].to_string());
    let mut show_create_word = use_signal(|| false);
    let mut create_word_front = use_signal(String::new);
    let mut create_word_back = use_signal(String::new);
    let mut create_word_pos = use_signal(String::new);
    let mut create_word_pron = use_signal(String::new);
    let mut create_word_example = use_signal(String::new);
    let mut create_word_synonym = use_signal(String::new);
    let mut create_word_antonym = use_signal(String::new);
    let mut create_word_tags = use_signal(String::new);
    let mut all_decks = use_signal(Vec::<Deck>::new);
    let mut selected_deck_idx = use_signal(|| None::<usize>);
    let pos_options = ["", "名詞", "動詞", "形容詞", "副詞", "介系詞", "連接詞", "代名詞", "感嘆詞", "片語", "其他"];

    use_effect(move || {
        let _ = *refresh.read();
        let _ = sort_by.cloned();
        let _ = *sort_asc.read();
        let _ = *folder_first.read();
        let fid = *current_folder_id.read();
        let db = app.db.cloned();
        spawn(async move {
            if let Some(db) = db {
                match db.list_by_parent(fid) {
                    Ok(list) => {
                        let sb = sort_by.cloned();
                        let sa = *sort_asc.read();
                        let ffirst = *folder_first.read();
                        let sort_key = |a: &Deck, b: &Deck| match sb.as_str() {
                            "name" => a.name.cmp(&b.name),
                            "count" => a.word_count.cmp(&b.word_count),
                            _ => a.updated_at.cmp(&b.updated_at).then_with(|| a.id.cmp(&b.id)),
                        };
                        let apply_dir = |cmp: std::cmp::Ordering| {
                            if sa { cmp } else { cmp.reverse() }
                        };
                        let mut folders: Vec<Deck> =
                            list.iter().filter(|d| d.is_folder).cloned().collect();
                        let mut decks: Vec<Deck> =
                            list.iter().filter(|d| !d.is_folder).cloned().collect();
                        folders.sort_by(|a, b| apply_dir(sort_key(a, b)));
                        decks.sort_by(|a, b| apply_dir(sort_key(a, b)));
                        let sorted: Vec<Deck> = if ffirst {
                            folders.into_iter().chain(decks).collect()
                        } else {
                            decks.into_iter().chain(folders).collect()
                        };
                        items.set(sorted);
                    }
                    Err(e) => log!("[Library] list_by_parent failed: {e}"),
                }
                match db.get_folder_path(fid) {
                    Ok(path) => breadcrumb.set(path),
                    Err(e) => log!("[Library] get_folder_path failed: {e}"),
                }
                match db.list_direct_decks(fid) {
                    Ok(decks) => all_decks.set(decks),
                    Err(e) => log!("[Library] list_direct_decks failed: {e}"),
                }
            }
        });
    });

    let item_vec = items.cloned();
    let breadcrumb_vec = breadcrumb.cloned();

    let search_q = search_text.read().clone();
    let display_items: Vec<GroupedItem> = if search_q.trim().is_empty() {
        item_vec.into_iter().map(GroupedItem::Deck).collect()
    } else {
        let db = app.db.read().clone();
        let Some(db) = db.as_ref() else {
            return rsx! { div { class: "library-screen" } };
        };
        let results = db.search_decks(&search_q).ok().unwrap_or_default();
        if results.is_empty() {
            item_vec.into_iter().map(GroupedItem::Deck).collect()
        } else {
            let mut groups: HashMap<Option<i64>, Vec<Deck>> = HashMap::new();
            for deck in results {
                groups.entry(deck.parent_id).or_default().push(deck);
            }
            // Load ALL folders in one query for in-memory path resolution.
            let folder_map: HashMap<i64, (String, Option<i64>)> = db
                .get_folder_map(&[])
                .ok()
                .unwrap_or_default();
            let sb = sort_by.cloned();
            let sa = *sort_asc.read();
            let sort_key = |a: &Deck, b: &Deck| match sb.as_str() {
                "name" => a.name.cmp(&b.name),
                "count" => a.word_count.cmp(&b.word_count),
                _ => a.updated_at.cmp(&b.updated_at).then_with(|| a.id.cmp(&b.id)),
            };
            let apply_dir = |cmp: std::cmp::Ordering| {
                if sa { cmp } else { cmp.reverse() }
            };
            let mut group_entries: Vec<(Option<i64>, Vec<Deck>)> = groups.drain().collect();
            for (_, decks) in &mut group_entries {
                decks.sort_by(|a, b| apply_dir(sort_key(a, b)));
            }
            group_entries.sort_by(|(parent_a, decks_a), (parent_b, decks_b)| {
                match (parent_a, parent_b) {
                    (None, None) => std::cmp::Ordering::Equal,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (Some(_), Some(_)) => match (decks_a.first(), decks_b.first()) {
                        (Some(da), Some(db)) => apply_dir(sort_key(da, db)),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    },
                }
            });
            let mut grouped: Vec<GroupedItem> = Vec::new();
            for (parent_id, decks) in group_entries {
                let name = match parent_id {
                    None => "根目錄".to_string(),
                    Some(pid) => {
                        // Resolve folder path from in-memory map (no per-group SQL).
                        let mut path = Vec::new();
                        let mut cur = Some(pid);
                        while let Some(fid) = cur {
                            if let Some((n, p)) = folder_map.get(&fid) {
                                path.push(n.clone());
                                cur = *p;
                            } else {
                                path.push("未知資料夾".to_string());
                                break;
                            }
                        }
                        path.reverse();
                        path.join(" / ")
                    }
                };
                grouped.push(GroupedItem::Header { name, count: decks.len() });
                for deck in decks {
                    grouped.push(GroupedItem::Deck(deck));
                }
            }
            grouped
        }
    };

    rsx! {
        div { class: "library-screen",
            button {
                class: "library-back",
                style: "position: absolute; opacity: 0; width: 1px; height: 1px; overflow: hidden;",
                onclick: move |_| {
                    let bc = breadcrumb.read().clone();
                    if !bc.is_empty() {
                        let parent_id = if bc.len() >= 2 {
                            Some(bc[bc.len() - 2].id)
                        } else {
                            None
                        };
                        current_folder_id.set(parent_id);
                    } else {
                        // 根目錄 → 走 JS 雙擊退出流程（與 Upload 畫面行為一致）
                        spawn(async move {
                            let js = r#"
                                if (!window.__backExitFlag) {
                                    window.__backExitFlag = true;
                                    try { AndroidBackHandler.showToast('再按一次以退出'); } catch(e) {}
                                    setTimeout(function() { window.__backExitFlag = false; }, 3000);
                                } else {
                                    try { AndroidBackHandler.finishActivity(); } catch(e) {}
                                }
                            "#;
                            let _ = document::eval(js).await;
                        });
                    }
                },
            }
            div { class: "library-topbar",
                div {
                    class: format!("breadcrumb{}", if *search_mode.read() { " search-mode" } else { "" }),
                    button {
                        class: "breadcrumb-btn breadcrumb-home",
                        onclick: move |_| current_folder_id.set(None),
                        span { class: "material-symbols-outlined", "folder" }
                    }
                    {breadcrumb_vec.into_iter().map(|f| {
                        let fid = f.id;
                        let fname = f.name.clone();
                        rsx! {
                            span { class: "breadcrumb-sep", ">" }
                            button {
                                class: "breadcrumb-btn",
                                onclick: move |_| current_folder_id.set(Some(fid)),
                                "{fname}"
                            }
                        }
                    })}
                }
                div {
                    class: format!("search-bar{}", if *search_mode.read() { " search-mode" } else { "" }),
                    span {
                        class: "material-symbols-outlined search-icon-inline",
                        "search"
                    }
                    input {
                        class: "search-input",
                        r#type: "text",
                        autocomplete: "off",
                        autocapitalize: "off",
                        autocorrect: "off",
                        spellcheck: "false",
                        enterkeyhint: "search",
                        lang: "zh-Hant-TW",
                        placeholder: "搜尋牌組、資料夾...",
                        oninput: move |e| { search_text.set(e.value()); },
                    }
                    button {
                        class: "search-close",
                        onclick: move |_| {
                            search_mode.set(false);
                            search_text.set(String::new());
                        },
                        span { class: "material-symbols-outlined", "close" }
                    }
                }
                if !*search_mode.read() {
                    button {
                        class: "search-btn",
                        onclick: move |_| search_mode.set(true),
                        span { class: "material-symbols-outlined", "search" }
                    }
                }
                if *show_sort_menu.read() {
                    div {
                        class: "sort-overlay",
                        onclick: move |_| show_sort_menu.set(false),
                    }
                }
                div { class: "sort-btn-wrap",
                    button {
                        class: "sort-btn",
                        onclick: move |_| {
                            let next = !*show_sort_menu.read();
                            show_sort_menu.set(next);
                        },
                        span { class: "material-symbols-outlined", "sort" }
                    }
                    if *show_sort_menu.read() {
                        div { class: "sort-menu open",
                            onclick: move |e| e.stop_propagation(),
                            div { class: "sort-menu-title", "排序方式" }
                            div { class: "sort-menu-section",
                                div { class: "sort-menu-label", "優先顯示" }
                                button {
                                    class: format!("sort-menu-item{}", if *folder_first.read() { " selected" } else { "" }),
                                    onclick: move |_| folder_first.set(true),
                                    span { class: "sort-menu-check material-symbols-outlined", "check" }
                                    "資料夾優先"
                                }
                                button {
                                    class: format!("sort-menu-item{}", if !*folder_first.read() { " selected" } else { "" }),
                                    onclick: move |_| folder_first.set(false),
                                    span { class: "sort-menu-check material-symbols-outlined", "check" }
                                    "牌組優先"
                                }
                            }
                            div { class: "sort-menu-section",
                                div { class: "sort-menu-label", "排序依據" }
                                button {
                                    class: format!("sort-menu-item{}", if *sort_by.read() == "name" { " selected" } else { "" }),
                                    onclick: move |_| sort_by.set("name".to_string()),
                                    span { class: "sort-menu-check material-symbols-outlined", "check" }
                                    "名稱"
                                }
                                button {
                                    class: format!("sort-menu-item{}", if *sort_by.read() == "date" { " selected" } else { "" }),
                                    onclick: move |_| sort_by.set("date".to_string()),
                                    span { class: "sort-menu-check material-symbols-outlined", "check" }
                                    "日期"
                                }
                                button {
                                    class: format!("sort-menu-item{}", if *sort_by.read() == "count" { " selected" } else { "" }),
                                    onclick: move |_| sort_by.set("count".to_string()),
                                    span { class: "sort-menu-check material-symbols-outlined", "check" }
                                    "詞數"
                                }
                            }
                            div { class: "sort-menu-section",
                                div { class: "sort-menu-label", "排序方向" }
                                button {
                                    class: format!("sort-menu-item{}", if *sort_asc.read() { " selected" } else { "" }),
                                    onclick: move |_| sort_asc.set(true),
                                    span { class: "sort-menu-check material-symbols-outlined", "check" }
                                    "↑ 升序"
                                }
                                button {
                                    class: format!("sort-menu-item{}", if !*sort_asc.read() { " selected" } else { "" }),
                                    onclick: move |_| sort_asc.set(false),
                                    span { class: "sort-menu-check material-symbols-outlined", "check" }
                                    "↓ 降序"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "deck-list",
                {display_items.into_iter().map(|gi| {
                    match gi {
                        GroupedItem::Header { name, count } => {
                            rsx! {
                                div { class: "search-group-header",
                                    span { class: "material-symbols-outlined search-group-icon", "folder" }
                                    span { class: "search-group-title", "{name}" }
                                    span { class: "search-group-count", "({count})" }
                                }
                            }
                        }
                        GroupedItem::Deck(item) => {
                            if item.is_folder {
                                let fid = item.id;
                                let fname = item.name.clone();
                                let fdate = item.updated_at.clone();
                                rsx! {
                                    div {
                                        key: "folder-{item.id}",
                                        class: "folder-item",
                                        onclick: move |_| current_folder_id.set(Some(fid)),
                                        span { class: "material-symbols-outlined folder-icon", "folder" }
                                        div { class: "deck-content",
                                            div { class: "deck-headline", "{fname}" }
                                        }
                                        div { class: "deck-actions",
                                            div { class: "deck-btns",
                                                button {
                                                    class: "deck-btn",
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        rename_target.set(fid);
                                                        rename_name.set(fname.clone());
                                                        is_folder_target.set(true);
                                                        show_rename.set(true);
                                                    },
                                                    span { class: "material-symbols-outlined", "edit" }
                                                }
                                                button {
                                                    class: "deck-btn danger",
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        delete_target.set(fid);
                                                        is_folder_target.set(true);
                                                        show_delete.set(true);
                                                    },
                                                    span { class: "material-symbols-outlined", "delete" }
                                                }
                                            }
                                            div { class: "deck-date", "{fdate}" }
                                        }
                                    }
                                }
                            } else {
                                let deck_id = item.id;
                                let deck_name = item.name.clone();
                                let color = item.color.clone();
                                let wc = item.word_count;
                                let ddate = item.updated_at.clone();
                                let did = deck_id;
                                rsx! {
                                    div {
                                        key: "deck-{item.id}",
                                        class: "deck-item",
                                        onclick: move |_| app.screen.set(Screen::DeckDetail { deck_id: did }),
                                        span { class: "deck-color-dot", style: "background: {color}" }
                                        div { class: "deck-content",
                                            div { class: "deck-headline", "{deck_name}" }
                                            div { class: "deck-supporting", "{wc} 詞" }
                                        }
                                        div { class: "deck-actions",
                                            div { class: "deck-btns",
                                                button {
                                                    class: "deck-btn",
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        rename_target.set(deck_id);
                                                        rename_name.set(deck_name.clone());
                                                        is_folder_target.set(false);
                                                        show_rename.set(true);
                                                    },
                                                    span { class: "material-symbols-outlined", "edit" }
                                                }
                                                button {
                                                    class: "deck-btn danger",
                                                    onclick: move |e| {
                                                        e.stop_propagation();
                                                        delete_target.set(deck_id);
                                                        is_folder_target.set(false);
                                                        show_delete.set(true);
                                                    },
                                                    span { class: "material-symbols-outlined", "delete" }
                                                }
                                            }
                                            div { class: "deck-date", "{ddate}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                })}
            }
        }
        if *show_fab_menu.read() {
            div {
                class: "fab-overlay",
                onclick: move |_| show_fab_menu.set(false),
                div { class: "fab-speed-dial",
                    button {
                        class: "fab-option",
                        onclick: move |_| {
                            show_fab_menu.set(false);
                            create_name.set(String::new());
                            selected_color.set(DECK_COLORS[0].to_string());
                            show_create_deck.set(true);
                        },
                        span { class: "material-symbols-outlined", "note_add" }
                        span { "牌組" }
                    }
                    button {
                        class: "fab-option",
                        onclick: move |_| {
                            show_fab_menu.set(false);
                            create_name.set(String::new());
                            show_create_folder.set(true);
                        },
                        span { class: "material-symbols-outlined", "create_new_folder" }
                        span { "資料夾" }
                    }
                    button {
                        class: "fab-option",
                        onclick: move |_| {
                            show_fab_menu.set(false);
                            create_word_front.set(String::new());
                            create_word_back.set(String::new());
                            create_word_pos.set(String::new());
                            create_word_pron.set(String::new());
                            create_word_example.set(String::new());
                            create_word_synonym.set(String::new());
                            create_word_antonym.set(String::new());
                            create_word_tags.set(String::new());
                            selected_deck_idx.set(None);
                            show_create_word.set(true);
                        },
                        span { class: "material-symbols-outlined", "playlist_add" }
                        span { "字彙" }
                    }
                }
            }
        }
        button {
            class: format!("add-fab{}", if *show_fab_menu.read() { " active" } else { "" }),
            onclick: move |_| {
                let next = !*show_fab_menu.read();
                show_fab_menu.set(next);
            },
            span { class: "material-symbols-outlined", if *show_fab_menu.read() { "close" } else { "add" } }
            }
            ModalDialog {
            visible: *show_create_deck.read(),
            title: "新增牌組",
            div { class: "update-body",
            input {
                class: "fsrs-input",
                placeholder: "牌組名稱",
                value: "{create_name}",
                oninput: move |e| create_name.set(e.value()),
            }
            div { class: "color-picker",
                div { class: "color-picker-label", "標記顏色" }
                div { class: "color-picker-grid",
                    {DECK_COLORS.into_iter().map(|c| {
                        let color = *c;
                        let selected = *selected_color.read() == color;
                        rsx! {
                            button {
                                class: format!("color-swatch{}", if selected { " selected" } else { "" }),
                                style: "background: {color}",
                                onclick: move |_| selected_color.set(color.to_string()),
                                if selected {
                                    span { class: "material-symbols-outlined", "check" }
                                }
                            }
                        }
                    })}
                }
            }
            }

            div { class: "update-actions",
            button {
                class: "update-btn secondary",
                onclick: move |_| show_create_deck.set(false),
                "取消"
            }
            button {
                class: "update-btn primary",
                onclick: move |_| {
                    let name = create_name.read().trim().to_string();
                    if name.is_empty() { return; }
                    let color = selected_color.read().clone();
                    let pid = *current_folder_id.read();
                    let db = app.db.cloned();
                    spawn(async move {
                        if let Some(db) = db {
                            match db.create_deck(&name, pid) {
                                Ok(d) => {
                                    let _ = db.update_deck_color(d.id, &color);
                                }
                                Err(e) => log!("[Library] create_deck failed: {e}"),
                            }
                            refresh.set(refresh() + 1);
                        }
                    });
                    show_create_deck.set(false);
                },
                "確定"
            }
            }
            }
            ModalDialog {
            visible: *show_create_folder.read(),
            title: "新增資料夾",
            div { class: "update-body",
            input {
                class: "fsrs-input",
                placeholder: "資料夾名稱",
                value: "{create_name}",
                oninput: move |e| create_name.set(e.value()),
            }
            }
            div { class: "update-actions",
            button {
                class: "update-btn secondary",
                onclick: move |_| show_create_folder.set(false),
                "取消"
            }
            button {
                class: "update-btn primary",
                onclick: move |_| {
                    let name = create_name.read().trim().to_string();
                    if name.is_empty() { return; }
                    let pid = *current_folder_id.read();
                    let db = app.db.cloned();
                    spawn(async move {
                        if let Some(db) = db {
                            if let Err(e) = db.create_folder(&name, pid) {
                                log!("[Library] create_folder failed: {e}");
                            }
                            refresh.set(refresh() + 1);
                        }
                    });
                    show_create_folder.set(false);
                },
                "確定"
            }
            }
            }
            ModalDialog {
            visible: *show_rename.read(),
            title: "重新命名",
            div { class: "update-body",
                input {
                    class: "fsrs-input",
                    placeholder: "名稱",
                    value: "{rename_name}",
                    oninput: move |e| rename_name.set(e.value()),
                }
            }
            div { class: "update-actions",
                button {
                    class: "update-btn secondary",
                    onclick: move |_| show_rename.set(false),
                    "取消"
                }
                button {
                    class: "update-btn primary",
                    onclick: move |_| {
                        let name = rename_name.read().trim().to_string();
                        let id = *rename_target.read();
                        if name.is_empty() { return; }
                        let db = app.db.cloned();
                        spawn(async move {
                            if let Some(db) = db {
                                if let Err(e) = db.rename_deck(id, &name) {
                                    log!("[Library] rename_deck failed: {e}");
                                }
                                refresh.set(refresh() + 1);
                            }
                        });
                        show_rename.set(false);
                    },
                    "確定"
                }
            }
        }
        ModalDialog {
            visible: *show_delete.read(),
            title: "刪除",
            div { class: "update-body",
                if *is_folder_target.read() {
                    "確定要刪除此資料夾？內含的所有牌組與子資料夾也將一併刪除。"
                } else {
                    "確定要刪除此牌組？牌組內的所有單字也將一併刪除。"
                }
            }
            div { class: "update-actions",
                button {
                    class: "update-btn secondary",
                    onclick: move |_| show_delete.set(false),
                    "取消"
                }
                button {
                    class: "update-btn primary",
                    onclick: move |_| {
                        let id = *delete_target.read();
                        let db = app.db.cloned();
                        spawn(async move {
                            if let Some(db) = db {
                                if let Err(e) = db.delete_deck(id) {
                                    log!("[Library] delete_deck failed: {e}");
                                }
                                refresh.set(refresh() + 1);
                            }
                        });
                        show_delete.set(false);
                    },
                    "確定"
                }
            }
        }
        ModalDialog {
            visible: *show_create_word.read(),
            title: "新增字彙",
            div { class: "update-body",
                select {
                    class: "word-form-select",
                    style: "margin-bottom: 12px;",
                    oninput: move |e| {
                        if let Ok(idx) = e.value().parse::<usize>() {
                            selected_deck_idx.set(Some(idx));
                        } else {
                            selected_deck_idx.set(None);
                        }
                    },
                    option {
                        value: "",
                        disabled: true,
                        selected: selected_deck_idx.read().is_none(),
                        "選擇牌組"
                    }
                    {all_decks.cloned().into_iter().enumerate().map(|(i, d)| {
                        let name = d.name.clone();
                        let sel = *selected_deck_idx.read() == Some(i);
                        rsx! {
                            option {
                                value: "{i}",
                                selected: sel,
                                "{name}"
                            }
                        }
                    })}
                }
                div { class: "word-form",
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "單字 *",
                            value: "{create_word_front}",
                            oninput: move |e| create_word_front.set(e.value()),
                        }
                    }
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "翻譯 *",
                            value: "{create_word_back}",
                            oninput: move |e| create_word_back.set(e.value()),
                        }
                    }
                    div { class: "word-form-row word-form-pos-row",
                        select {
                            class: "word-form-select",
                            value: "{create_word_pos}",
                            oninput: move |e| create_word_pos.set(e.value()),
                            {pos_options.into_iter().map(|p| {
                                let val = p;
                                rsx! {
                                    option {
                                        value: "{val}",
                                        selected: *create_word_pos.read() == val,
                                        if val.is_empty() { "詞性" } else { "{val}" }
                                    }
                                }
                            })}
                        }
                        input {
                            class: "word-form-input",
                            placeholder: "發音",
                            value: "{create_word_pron}",
                            oninput: move |e| create_word_pron.set(e.value()),
                        }
                    }
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "例句",
                            value: "{create_word_example}",
                            oninput: move |e| create_word_example.set(e.value()),
                        }
                    }
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "同義詞",
                            value: "{create_word_synonym}",
                            oninput: move |e| create_word_synonym.set(e.value()),
                        }
                        input {
                            class: "word-form-input",
                            placeholder: "反義詞",
                            value: "{create_word_antonym}",
                            oninput: move |e| create_word_antonym.set(e.value()),
                        }
                    }
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "標籤 (逗號分隔)",
                            value: "{create_word_tags}",
                            oninput: move |e| create_word_tags.set(e.value()),
                        }
                    }
                }
            }
            div { class: "update-actions",
                button {
                    class: "update-btn secondary",
                    onclick: move |_| show_create_word.set(false),
                    "取消"
                }
                button {
                    class: "update-btn primary",
                    disabled: selected_deck_idx.read().is_none() || all_decks.read().is_empty(),
                    onclick: move |_| {
                        let front = create_word_front.read().trim().to_string();
                        let back = create_word_back.read().trim().to_string();
                        if front.is_empty() || back.is_empty() { return; }
                        let decks = all_decks.cloned();
                        let Some(idx) = *selected_deck_idx.read() else { return; };
                        if idx >= decks.len() { return; }
                        let deck_id = decks[idx].id;
                        let pos = create_word_pos.read().clone();
                        let pron = create_word_pron.read().clone();
                        let example = create_word_example.read().clone();
                        let synonym = create_word_synonym.read().clone();
                        let antonym = create_word_antonym.read().clone();
                        let tags_input = create_word_tags.read().clone();
                        let tags_json = if tags_input.is_empty() {
                            "[]".to_string()
                        } else {
                            let parts: Vec<String> = tags_input.split(',').map(|s| format!("\"{}\"", s.trim())).collect();
                            format!("[{}]", parts.join(","))
                        };
                        let db = app.db.cloned();
                        spawn(async move {
                            if let Some(db) = db {
                                let _ = db.add_word(deck_id, &front, &back, &pos, &pron, &example, &synonym, &antonym, &tags_json);
                                refresh.set(refresh() + 1);
                            }
                        });
                        show_create_word.set(false);
                    },
                    "確定"
                }
            }
        }
    }
}

const POS_OPTIONS: &[&str] = &["", "名詞", "動詞", "形容詞", "副詞", "介系詞", "連接詞", "代名詞", "感嘆詞", "片語", "其他"];

#[component]
fn DeckDetailScreen() -> Element {
    let mut app = use_context::<AppSignals>();
    let deck_id = match *app.screen.read() {
        Screen::DeckDetail { deck_id } => deck_id,
        _ => return rsx! { div {} },
    };
    let mut refresh = use_signal(|| 0u64);

    let deck = use_resource(move || {
        let _ = *refresh.read();
        let db = app.db.cloned();
        let did = deck_id;
        async move {
            db.as_ref().and_then(|db| db.get_deck(did).ok())
        }
    });
    let words = use_resource(move || {
        let _ = *refresh.read();
        let db = app.db.cloned();
        let did = deck_id;
        async move {
            db.as_ref().and_then(|db| db.list_words_by_deck(did).ok())
        }
    });

    let deck_data = deck.read_unchecked().clone().flatten();
    let word_list = words.read_unchecked().clone().flatten().unwrap_or_default();

    let deck_name = deck_data.as_ref().map(|d| d.name.clone()).unwrap_or_default();
    let word_count = deck_data.as_ref().map(|d| d.word_count).unwrap_or(0);

    let mut show_edit = use_signal(|| false);
    let mut edit_id = use_signal(|| 0i64);
    let mut edit_front = use_signal(String::new);
    let mut edit_back = use_signal(String::new);
    let mut edit_pos = use_signal(String::new);
    let mut edit_pron = use_signal(String::new);
    let mut edit_example = use_signal(String::new);
    let mut edit_synonym = use_signal(String::new);
    let mut edit_antonym = use_signal(String::new);
    let mut edit_tags = use_signal(String::new);
    let mut show_delete_confirm = use_signal(|| false);
    let mut delete_target = use_signal(|| 0i64);

    rsx! {
        div { class: "deck-detail-screen",
            div { class: "deck-detail-topbar",
                button {
                    class: "deck-detail-back",
                    onclick: move |_| app.screen.set(Screen::Library),
                    span { class: "material-symbols-outlined", "arrow_back" }
                }
                div { class: "deck-detail-title",
                    div { class: "deck-detail-name", "{deck_name}" }
                    div { class: "deck-detail-count", "{word_count} 詞" }
                }
            }
            if word_list.is_empty() {
                div { class: "deck-detail-empty",
                    span { class: "material-symbols-outlined", "playlist_remove" }
                    span { "尚無字彙" }
                }
            } else {
                div { class: "deck-detail-list",
                    {word_list.into_iter().map(|(wid, w)| {
                        rsx! {
                            div { class: "deck-word-item",
                                div { class: "word-main",
                                    div { class: "word-front", "{w.front}" }
                                    div { class: "word-back", "{w.back}" }
                                    div { class: "word-meta",
                                        {(!w.pos.is_empty()).then(|| rsx! {
                                            span { class: "word-pos-tag", "{w.pos}" }
                                        })}
                                        {(!w.pron.is_empty()).then(|| rsx! {
                                            span { class: "word-pron", "{w.pron}" }
                                        })}
                                    }
                                    {(!w.example.is_empty()).then(|| rsx! {
                                        div { class: "word-example-row",
                                            span { class: "word-field-label", "例句" }
                                            div { class: "word-example-scroll",
                                                span { "{w.example}" }
                                            }
                                        }
                                    })}
                                    {(!w.synonym.is_empty()).then(|| rsx! {
                                        div { class: "word-field-row",
                                            span { class: "word-field-label", "同義詞" }
                                            span { "{w.synonym}" }
                                        }
                                    })}
                                    {(!w.antonym.is_empty()).then(|| rsx! {
                                        div { class: "word-field-row",
                                            span { class: "word-field-label", "反義詞" }
                                            span { "{w.antonym}" }
                                        }
                                    })}
                                }
                                button {
                                    class: "word-edit-btn",
                                    onclick: move |_| {
                                        edit_id.set(wid);
                                        edit_front.set(w.front.clone());
                                        edit_back.set(w.back.clone());
                                        edit_pos.set(w.pos.clone());
                                        edit_pron.set(w.pron.clone());
                                        edit_example.set(w.example.clone());
                                        edit_synonym.set(w.synonym.clone());
                                        edit_antonym.set(w.antonym.clone());
                                        let tag_str = w.tags.join(", ");
                                        edit_tags.set(tag_str);
                                        show_edit.set(true);
                                    },
                                    span { class: "material-symbols-outlined", "edit" }
                                }
                            }
                        }
                    })}
                }
            }
        }

        ModalDialog {
            visible: *show_edit.read(),
            title: "編輯字彙",
            div { class: "update-body",
                div { class: "word-form",
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "單字 *",
                            value: "{edit_front}",
                            oninput: move |e| edit_front.set(e.value()),
                        }
                    }
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "翻譯 *",
                            value: "{edit_back}",
                            oninput: move |e| edit_back.set(e.value()),
                        }
                    }
                    div { class: "word-form-row word-form-pos-row",
                        select {
                            class: "word-form-select",
                            value: "{edit_pos}",
                            oninput: move |e| edit_pos.set(e.value()),
                            {POS_OPTIONS.into_iter().map(|p| {
                                let val = p;
                                rsx! {
                                    option {
                                        value: "{val}",
                                        selected: *edit_pos.read() == *val,
                                        if val.is_empty() { "詞性" } else { "{val}" }
                                    }
                                }
                            })}
                        }
                        input {
                            class: "word-form-input",
                            placeholder: "發音",
                            value: "{edit_pron}",
                            oninput: move |e| edit_pron.set(e.value()),
                        }
                    }
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "例句",
                            value: "{edit_example}",
                            oninput: move |e| edit_example.set(e.value()),
                        }
                    }
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "同義詞",
                            value: "{edit_synonym}",
                            oninput: move |e| edit_synonym.set(e.value()),
                        }
                        input {
                            class: "word-form-input",
                            placeholder: "反義詞",
                            value: "{edit_antonym}",
                            oninput: move |e| edit_antonym.set(e.value()),
                        }
                    }
                    div { class: "word-form-row",
                        input {
                            class: "word-form-input",
                            placeholder: "標籤 (逗號分隔)",
                            value: "{edit_tags}",
                            oninput: move |e| edit_tags.set(e.value()),
                        }
                    }
                }
            }
            div { class: "dialog-actions-row",
                button {
                    class: "dialog-btn-text danger",
                    onclick: move |_| {
                        delete_target.set(*edit_id.read());
                        show_edit.set(false);
                        show_delete_confirm.set(true);
                    },
                    span { class: "material-symbols-outlined", "delete" }
                    " 刪除"
                }
                div { class: "spacer" }
                button {
                    class: "dialog-btn-text",
                    onclick: move |_| show_edit.set(false),
                    "取消"
                }
                button {
                    class: "dialog-btn-filled",
                    onclick: move |_| {
                        let front = edit_front.read().trim().to_string();
                        let back = edit_back.read().trim().to_string();
                        if front.is_empty() || back.is_empty() { return; }
                        let wid = *edit_id.read();
                        let pos = edit_pos.read().clone();
                        let pron = edit_pron.read().clone();
                        let example = edit_example.read().clone();
                        let synonym = edit_synonym.read().clone();
                        let antonym = edit_antonym.read().clone();
                        let tags_input = edit_tags.read().clone();
                        let tags_json = if tags_input.is_empty() {
                            "[]".to_string()
                        } else {
                            let parts: Vec<String> = tags_input.split(',').map(|s| format!("\"{}\"", s.trim())).collect();
                            format!("[{}]", parts.join(","))
                        };
                        let db = app.db.cloned();
                        spawn(async move {
                            if let Some(db) = db {
                                let _ = db.update_word(wid, &front, &back, &pos, &pron, &example, &synonym, &antonym, &tags_json);
                                refresh.set(refresh() + 1);
                            }
                        });
                        show_edit.set(false);
                    },
                    "儲存"
                }
            }
        }

        ModalDialog {
            visible: *show_delete_confirm.read(),
            title: "刪除字彙",
            div { class: "update-body", "確定要刪除此字彙？" }
            div { class: "update-actions",
                button {
                    class: "update-btn secondary",
                    onclick: move |_| show_delete_confirm.set(false),
                    "取消"
                }
                button {
                    class: "update-btn primary",
                    onclick: move |_| {
                        let wid = *delete_target.read();
                        let db = app.db.cloned();
                        spawn(async move {
                            if let Some(db) = db {
                                let _ = db.delete_word(wid);
                                refresh.set(refresh() + 1);
                            }
                        });
                        show_delete_confirm.set(false);
                    },
                    "確定"
                }
            }
        }
    }
}

#[component]
fn SettingsScreen() -> Element {
    let mut app = use_context::<AppSignals>();
    let mut settings_tab = use_signal(|| 0);
    let mut show_licenses = use_signal(|| false);
    let mut selected_dep: Signal<Option<(String, String)>> = use_signal(|| None);
    let show_detail = selected_dep.read().is_some();
    let detail_name = selected_dep.read().as_ref().map(|(n, _)| n.clone()).unwrap_or_default();
    let detail_text = selected_dep.read().as_ref().map(|(_, l)| licenses::get_license_text(l).to_owned()).unwrap_or_default();

    rsx! {
        div { class: "settings-screen",
            div { class: "settings-topbar",
                span { class: "settings-topbar-title", "設定" }
                button {
                    class: "settings-topbar-btn",
                    title: "還原預設值",
                    onclick: move |_| app.show_reset_confirm.set(true),
                    span { class: "material-symbols-outlined", "restart_alt" }
                }
                button {
                    class: "settings-topbar-btn",
                    title: "開源許可證",
                    onclick: move |_| show_licenses.set(true),
                    span { class: "material-symbols-outlined", "description" }
                }
                button {
                    class: "settings-topbar-btn",
                    title: "GitHub",
                    onclick: move |_| {
                        let js = r#"(function(){
                            if (window.AndroidExternal && typeof window.AndroidExternal.openUrl === 'function') {
                                window.AndroidExternal.openUrl("https://github.com/HongyiHank/scallion-vocab");
                            } else {
                                window.open("https://github.com/HongyiHank/scallion-vocab", "_blank", "noopener,noreferrer");
                            }
                        })()"#;
                        spawn(async move { let _ = document::eval(js).await; });
                    },
                    svg { width: "24", height: "24", view_box: "0 0 98 96",
                        path {
                            fill: "currentColor",
                            d: "M41.4395 69.3848C28.8066 67.8535 19.9062 58.7617 19.9062 46.9902C19.9062 42.2051 21.6289 37.0371 24.5 33.5918C23.2559 30.4336 23.4473 23.7344 24.8828 20.959C28.7109 20.4805 33.8789 22.4902 36.9414 25.2656C40.5781 24.1172 44.4062 23.543 49.0957 23.543C53.7852 23.543 57.6133 24.1172 61.0586 25.1699C64.0254 22.4902 69.2891 20.4805 73.1172 20.959C74.457 23.543 74.6484 30.2422 73.4043 33.4961C76.4668 37.1328 78.0937 42.0137 78.0937 46.9902C78.0937 58.7617 69.1934 67.6621 56.3691 69.2891C59.623 71.3945 61.8242 75.9883 61.8242 81.252L61.8242 91.2051C61.8242 94.0762 64.2168 95.7031 67.0879 94.5547C84.4102 87.9512 98 70.6289 98 49.1914C98 22.1074 75.9883 6.69539e-07 48.9043 4.309e-07C21.8203 1.92261e-07 -1.9479e-07 22.1074 -4.3343e-07 49.1914C-6.20631e-07 70.4375 13.4941 88.0469 31.6777 94.6504C34.2617 95.6074 36.75 93.8848 36.75 91.3008L36.75 83.6445C35.4102 84.2188 33.6875 84.6016 32.1562 84.6016C25.8398 84.6016 22.1074 81.1563 19.4277 74.7441C18.375 72.1602 17.2266 70.6289 15.0254 70.3418C13.877 70.2461 13.4941 69.7676 13.4941 69.1934C13.4941 68.0449 15.4082 67.1836 17.3223 67.1836C20.0977 67.1836 22.4902 68.9063 24.9785 72.4473C26.8926 75.2227 28.9023 76.4668 31.2949 76.4668C33.6875 76.4668 35.2187 75.6055 37.4199 73.4043C39.0469 71.7773 40.291 70.3418 41.4395 69.3848Z",
                        }
                    }
                }
            }
            div { class: "settings-tabs",
                button {
                    class: if *settings_tab.read() == 0 { "settings-tab active" } else { "settings-tab" },
                    onclick: move |_| settings_tab.set(0),
                    "一般"
                }
                button {
                    class: if *settings_tab.read() == 1 { "settings-tab active" } else { "settings-tab" },
                    onclick: move |_| settings_tab.set(1),
                    "考試"
                }
                button {
                    class: if *settings_tab.read() == 2 { "settings-tab active" } else { "settings-tab" },
                    onclick: move |_| settings_tab.set(2),
                    "FSRS"
                }
            }
            if *settings_tab.read() == 0 {
                div { class: "settings-body",
                    div { class: "settings-section-label", "主題" }
                    div { class: "theme-segmented",
                        button {
                            class: if *app.theme_mode.read() == ThemeMode::Light { "theme-btn active" } else { "theme-btn" },
                            onclick: move |_| app.theme_mode.set(ThemeMode::Light),
                            span { class: "material-symbols-outlined", "light_mode" }
                            span { "淺色" }
                        }
                        button {
                            class: if *app.theme_mode.read() == ThemeMode::System { "theme-btn active" } else { "theme-btn" },
                            onclick: move |_| app.theme_mode.set(ThemeMode::System),
                            span { class: "material-symbols-outlined", "settings_brightness" }
                            span { "系統" }
                        }
                        button {
                            class: if *app.theme_mode.read() == ThemeMode::Dark { "theme-btn active" } else { "theme-btn" },
                            onclick: move |_| app.theme_mode.set(ThemeMode::Dark),
                            span { class: "material-symbols-outlined", "dark_mode" }
                            span { "深色" }
                        }
                    }
                    div {
                        class: "settings-item",
                        onclick: move |_| {
                            let new_val = !*app.update_check_enabled.read();
                            app.update_check_enabled.set(new_val);
                        },
                        div { class: "settings-item-icon",
                            span { class: "material-symbols-outlined", "system_update" }
                        }
                        div { class: "settings-item-label", "更新檢測" }
                        div {
                            class: if *app.update_check_enabled.read() { "settings-switch on" } else { "settings-switch" },
                        }
                    }
                    div {
                        class: "settings-item",
                        onclick: move |_| {
                            spawn(async move {
                                let js = format!(
                                    r#"fetch('https://api.github.com/repos/{repo}/releases/latest',{{headers:{{'Accept':'application/json','User-Agent':'scallion-vocab'}}}}).then(r=>r.json()).then(d=>{{var tag=d.tag_name||'';var info=JSON.stringify({{tag:tag,url:(d.assets&&d.assets[0])?d.assets[0].browser_download_url:'',size:(d.assets&&d.assets[0])?d.assets[0].size:0}});dioxus.send(info)}}).catch(function(){{dioxus.send('')}});"#,
                                    repo = GH_REPO
                                );
                                let mut eval = document::eval(&js);
                                match eval.recv::<String>().await {
                                    Ok(json) if !json.is_empty() => {
                                        if let Ok(info) = serde_json::from_str::<UpdateInfo>(&json) {
                                            if !info.tag.is_empty()
                                                && !info.url.is_empty()
                                                && parse_version(&info.tag).map_or(false, |v| {
                                                    parse_version(APP_VERSION).map_or(true, |cur| v > cur)
                                                })
                                            {
                                                app.update_info.set(Some(info));
                                                return;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                push_toast(app, "已是最新版本");
                            });
                        },
                        div { class: "settings-item-icon",
                            span { class: "material-symbols-outlined", "update" }
                        }
                        div { class: "settings-item-label", "檢查更新" }
                    }
                }
            } else if *settings_tab.read() == 1 {
                div { class: "settings-body",
                    div {
                        class: "settings-item",
                        onclick: move |_| {
                            let new_val = !*app.infinite_mode.read();
                            app.infinite_mode.set(new_val);
                        },
                        div { class: "settings-item-icon",
                            span { class: "material-symbols-outlined", "all_inclusive" }
                        }
                        div { class: "settings-item-label", "無限考試" }
                        div {
                            class: if *app.infinite_mode.read() { "settings-switch on" } else { "settings-switch" },
                        }
                    }
                    div {
                        class: "settings-item",
                        onclick: move |_| {
                            let mut c = app.fsrs_config.cloned();
                            c.review_wrong = !c.review_wrong;
                            app.fsrs_config.set(c);
                        },
                        div { class: "settings-item-icon",
                            span { class: "material-symbols-outlined", "refresh" }
                        }
                        div { class: "settings-item-label",
                            div { "重複出現錯題" }
                            div { class: "settings-item-sub", "關閉時錯題不加入複習佇列，優先於 FSRS 設定" }
                        }
                        div {
                            class: if app.fsrs_config.read().review_wrong { "settings-switch on" } else { "settings-switch" },
                        }
                    }
                    div {
                        class: "settings-item",
                        onclick: move |_| {
                            let new_val = !*app.show_finished_screen.read();
                            app.show_finished_screen.set(new_val);
                        },
                        div { class: "settings-item-icon",
                            span { class: "material-symbols-outlined", "celebration" }
                        }
                        div { class: "settings-item-label",
                            div { "是否啟用結算分數" }
                            div { class: "settings-item-sub", "此設定僅在關閉無限考試時生效" }
                        }
                        div {
                            class: if *app.show_finished_screen.read() { "settings-switch on" } else { "settings-switch" },
                        }
                    }
                    div {
                        class: "settings-item",
                        style: "cursor: default;",
                        div { class: "settings-item-icon",
                            span { class: "material-symbols-outlined", "timer" }
                        }
                        div { class: "settings-item-label",
                            div { "自動跳題時間" }
                            div { class: "settings-item-sub", "設為負數則關閉" }
                        }
                        input {
                            class: "fsrs-input",
                            style: "width: 100px; flex-shrink: 0; text-align: right;",
                            r#type: "number",
                            value: "{app.auto_advance_ms.read()}",
                            oninput: move |e| {
                                let v = e.value().trim().to_string();
                                if v.is_empty() { return; }
                                if let Ok(n) = v.parse::<i64>() {
                                    app.auto_advance_ms.set(n);
                                }
                            },
                        }
                    }
                }
            } else {
                div { class: "settings-body",
                    FsrsSettings {}
                }
            }
            div { class: "settings-version", "v{APP_VERSION}" }
        }
        if *show_licenses.read() {
            div { class: "license-overlay",
                div { class: "license-dialog",
                    div { class: "license-dialog-topbar",
                        button {
                            class: "license-dialog-close",
                            onclick: move |_| show_licenses.set(false),
                            span { class: "material-symbols-outlined", "close" }
                        }
                        span { class: "license-dialog-title", "開源許可證" }
                    }
                    div { class: "license-list",
                        {licenses::ALL_DEPS.iter().map(|dep| {
                            let n = dep.name.to_owned();
                            let lf = dep.license_file.to_owned();
                            rsx! {
                                button {
                                    key: "{dep.name}",
                                    class: "license-item",
                                    onclick: move |_| {
                                        selected_dep.set(Some((n.clone(), lf.clone())));
                                    },
                                    span { class: "license-item-name", "{dep.name}" }
                                    span { class: "license-item-type", "{dep.license_display}" }
                                }
                            }
                        })}
                    }
                }
            }
        }
        {show_detail.then(|| {
            let n = detail_name.clone();
            let t = detail_text.clone();
            rsx! {
                div { class: "license-detail-overlay",
                    div { class: "license-detail-dialog",
                        div { class: "license-detail-topbar",
                            button {
                                class: "license-detail-close",
                                onclick: move |_| {
                                    selected_dep.set(None);
                                },
                                span { class: "material-symbols-outlined", "arrow_back" }
                            }
                            span { class: "license-detail-title", "{n}" }
                        }
                        div { class: "license-detail-body", "{t}" }
                    }
                }
            }
        })}
    }
}


#[component]
fn ImportScreen() -> Element {
    let mut app = use_context::<AppSignals>();
    let mut tab = use_signal(|| 0usize);
    let mut paste_text = use_signal(String::new);
    let mut url_text = use_signal(String::new);
    let mut parsed = use_signal::<Option<(Vec<Word>, String)>>(|| None);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut selected_deck_id = use_signal::<Option<i64>>(|| None);
    let mut show_create_deck = use_signal(|| false);
    let mut create_deck_name = use_signal(String::new);
    let mut file_name = use_signal::<Option<String>>(|| None);
    let mut fetch_err = use_signal(String::new);

    // 目標牌組資料夾瀏覽
    let mut dest_folder = use_signal::<Option<i64>>(|| None);
    let mut dest_items = use_signal(Vec::<Deck>::new);
    let mut dest_breadcrumb = use_signal(Vec::<Deck>::new);
    let mut dest_refresh = use_signal(|| 0u64);
    use_effect(move || {
        let _ = *dest_refresh.read();
        let fid = *dest_folder.read();
        let db = app.db.cloned();
        spawn(async move {
            let Some(db) = db else { return };
            if let Ok(list) = db.list_by_parent(fid) {
                let mut folders: Vec<Deck> = list.iter().filter(|d| d.is_folder).cloned().collect();
                let mut decks: Vec<Deck> = list.iter().filter(|d| !d.is_folder).cloned().collect();
                folders.sort_by(|a, b| a.name.cmp(&b.name));
                decks.sort_by(|a, b| a.name.cmp(&b.name));
                dest_items.set(folders.into_iter().chain(decks).collect());
            }
            if let Ok(bc) = db.get_folder_path(fid) {
                dest_breadcrumb.set(bc);
            }
        });
    });

    let mut choose_file = move |accept: &str| {
        loading.set(true);
        let accept = accept.to_string();
        spawn(async move {
            let js = format!(
                r#"let i=document.createElement('input');i.type='file';i.accept='{accept}';
                i.onchange=async()=>{{try{{let f=i.files[0];if(!f){{dioxus.send('')}}else{{let b=await f.arrayBuffer();let a=new Uint8Array(b);dioxus.send(JSON.stringify({{name:f.name,data:Array.from(a)}}))}}}}
                catch(e){{dioxus.send('')}}}};
                i.addEventListener('cancel',()=>dioxus.send(''));i.click();
                setTimeout(()=>dioxus.send(''),60000);"#
            );
            let mut eval = document::eval(&js);
            match eval.recv::<String>().await {
                Ok(json) if !json.is_empty() => {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json) {
                        let name = val["name"].as_str().unwrap_or("");
                        if let Some(arr) = val["data"].as_array() {
                            let bytes: Vec<u8> = arr.iter().filter_map(|v| v.as_u64()).map(|v| v as u8).collect();
                            file_name.set(Some(name.to_string()));
                            let result = if name.ends_with(".apkg") { parse_apkg(&bytes) } else { Ok(parse_anki_text(&String::from_utf8_lossy(&bytes))) };
                            match result {
                                Ok(words) if !words.is_empty() => {
                                    let label = format!("「{}」— {} 張卡片", name, words.len());
                                    parsed.set(Some((words, label)));
                                    error.set(String::new());
                                }
                                _ => error.set("檔案中無有效卡片".to_string()),
                            }
                        }
                    }
                }
                _ => error.set("已取消選擇檔案".to_string()),
            }
            loading.set(false);
        });
    };

    let mut parse_paste = move || {
        let text = paste_text.read().trim().to_string();
        if text.is_empty() { error.set("請輸入或貼上 Anki 文字".to_string()); return; }
        let words = parse_anki_text(&text);
        if words.is_empty() { error.set("無法解析任何卡片，請確認格式為「英文\\t中文」".to_string()); return; }
        let label = format!("手動輸入 — {} 張卡片", words.len());
        parsed.set(Some((words, label)));
        error.set(String::new());
    };

    let mut parse_url = move || {
        let urls = parse_quizlet_urls(&url_text.read());
        if urls.is_empty() { error.set("請輸入有效的 Quizlet 網址".to_string()); return; }
        loading.set(true);
        fetch_err.set(String::new());
        spawn(async move {
            let (all_words, errors) = fetch_quizlet_multi(&urls).await;
            if all_words.is_empty() {
                fetch_err.set(errors.join("\n"));
            } else {
                let label = format!("Quizlet — {} 張卡片", all_words.len());
                parsed.set(Some((all_words, label)));
                error.set(String::new());
                let mut recent = app.recent_urls.cloned();
                for u in &urls {
                    recent.retain(|x| x != u);
                    recent.insert(0, u.clone());
                }
                recent.truncate(MAX_RECENT_URLS);
                app.recent_urls.set(recent.clone());
                save_recent_urls(&recent).await;
            }
            loading.set(false);
        });
    };

    let mut do_import = move || {
        let Some((ref words, _)) = *parsed.read() else { return };
        if words.is_empty() { return; }
        let deck_id = *selected_deck_id.read();
        let deck_name = create_deck_name.read().trim().to_string();
        if deck_id.is_none() && deck_name.is_empty() { error.set("請選擇目標牌組或輸入新牌組名稱".to_string()); return; }
        loading.set(true);
        let words = words.clone();
        let db = app.db.cloned();
        let folder = *dest_folder.read();
        spawn(async move {
            let Some(db) = db else { loading.set(false); return };
            let target_id = match deck_id {
                Some(id) => id,
                None => match db.create_deck(&deck_name, folder) {
                    Ok(d) => d.id,
                    Err(_) => { loading.set(false); error.set("無法建立牌組".to_string()); return; }
                },
            };
            let mut ok = 0usize;
            let mut fail = 0usize;
            for w in &words {
                if db.add_word(target_id, &w.front, &w.back, "", "", "", "", "", "[]").is_ok() {
                    ok += 1;
                } else {
                    fail += 1;
                }
            }
            loading.set(false);
            parsed.set(None);
            selected_deck_id.set(None);
            show_create_deck.set(false);
            create_deck_name.set(String::new());
            file_name.set(None);
            dest_refresh.set(dest_refresh() + 1);
            error.set(String::new());
            let msg = if fail > 0 {
                format!("已匯入 {ok} 張，{fail} 張失敗")
            } else {
                format!("已匯入 {ok} 張卡片")
            };
            push_toast(app, msg);
        });
    };

    // Clone short-lived reads so async writes from use_effect are visible next frame
    let dest_bc = dest_breadcrumb.read().clone();
    let dest_items_vec = dest_items.read().clone();

    rsx! {
        div { class: "import-wrapper",
            section { class: "import-container",
                h2 { "匯入" }
                p { class: "import-subtitle", "從 Anki 或 Quizlet 匯入單字到字庫" }

                div { class: "import-tabs",
                    button {
                        class: if *tab.read() == 0 { "import-tab active" } else { "import-tab" },
                        onclick: move |_| { tab.set(0); parsed.set(None); error.set(String::new()); file_name.set(None); },
                        "Anki 文字檔"
                    }
                    button {
                        class: if *tab.read() == 1 { "import-tab active" } else { "import-tab" },
                        onclick: move |_| { tab.set(1); parsed.set(None); error.set(String::new()); file_name.set(None); },
                        "Anki 牌組包"
                    }
                    button {
                        class: if *tab.read() == 2 { "import-tab active" } else { "import-tab" },
                        onclick: move |_| { tab.set(2); parsed.set(None); error.set(String::new()); file_name.set(None); },
                        "Quizlet"
                    }
                }

                if *tab.read() == 0 {
                    div { class: "tab-panel",
                        div { class: "import-source",
                            button {
                                class: "file-picker",
                                onclick: move |_| choose_file(".txt"),
                                div { class: "file-picker-icon",
                                    span { class: "material-symbols-outlined", "upload_file" }
                                }
                                div { class: "file-picker-text",
                                    div { class: "file-picker-title", "選擇 .txt 檔案" }
                                    div { class: "file-picker-sub", "Tab 分隔的 Anki 導出格式" }
                                }
                                span { class: "material-symbols-outlined", style: "color: var(--md-sys-color-on-surface-variant)", "chevron_right" }
                            }
                            {file_name.read().as_ref().filter(|_| *tab.read() == 0).map(|n| rsx! {
                                div { class: "file-chip",
                                    span { class: "material-symbols-outlined", "check" }
                                    span { "{n}" }
                                }
                            })}
                            div { class: "divider-or", "或手動輸入" }
                            div { class: "section-label",
                                span { class: "material-symbols-outlined", "edit_note" }
                                "貼上 Anki 文字"
                            }
                            div { class: "md3-field",
                                textarea {
                                    rows: "5",
                                    placeholder: "英文\t中文\nhello\t你好\nworld\t世界",
                                    value: "{paste_text}",
                                    oninput: move |e| paste_text.set(e.value()),
                                }
                            }
                            div { class: "parse-row",
                                button {
                                    class: "md3-btn md3-btn--tonal",
                                    onclick: move |_| {
                                        if file_name.read().is_some() {
                                            choose_file(".txt,.csv,.tsv");
                                        } else {
                                            parse_paste();
                                        }
                                    },
                                    span { class: "material-symbols-outlined", "play_arrow" }
                                    "解析預覽"
                                }
                            }
                        }
                    }
                } else if *tab.read() == 1 {
                    div { class: "tab-panel",
                        div { class: "import-source",
                            button {
                                class: "file-picker",
                                onclick: move |_| choose_file(".apkg"),
                                div { class: "file-picker-icon",
                                    span { class: "material-symbols-outlined", "inventory_2" }
                                }
                                div { class: "file-picker-text",
                                    div { class: "file-picker-title", "選擇 .apkg 檔案" }
                                    div { class: "file-picker-sub", "Anki 牌組包格式，含完整卡片結構" }
                                }
                                span { class: "material-symbols-outlined", style: "color: var(--md-sys-color-on-surface-variant)", "chevron_right" }
                            }
                            {file_name.read().as_ref().filter(|_| *tab.read() == 1).map(|n| rsx! {
                                div { class: "file-chip",
                                    span { class: "material-symbols-outlined", "check" }
                                    span { "{n}" }
                                }
                            })}
                        }
                    }
                } else {
                    div { class: "tab-panel",
                        div { class: "import-source",
                            div { class: "section-label",
                                span { class: "material-symbols-outlined", "link" }
                                "Quizlet 網址"
                            }
                            div { class: "md3-field",
                                textarea {
                                    rows: "4",
                                    placeholder: "https://quizlet.com/123/deck/\nhttps://quizlet.com/456/flash-cards/",
                                    value: "{url_text}",
                                    oninput: move |e| url_text.set(e.value()),
                                }
                            }
                            if !fetch_err.read().is_empty() {
                                div { class: "error-banner", "{fetch_err.read().clone()}" }
                            }
                            div { class: "parse-row",
                                button {
                                    class: "md3-btn md3-btn--tonal",
                                    disabled: loading(),
                                    onclick: move |_| parse_url(),
                                    span { class: "material-symbols-outlined", "travel_explore" }
                                    if loading() { "抓取中…" } else { "抓取並預覽" }
                                }
                            }
                        }
                    }
                }

                if let Some((ref words, ref label)) = *parsed.read() {
                    div { class: "md3-card",
                        div { class: "md3-card-header",
                            span { class: "md3-card-title", "{label}" }
                            button {
                                class: "md3-card-close",
                                onclick: move |_| { parsed.set(None); error.set(String::new()); },
                                span { class: "material-symbols-outlined", "close" }
                            }
                        }
                        if words.len() > 50 {
                            div { class: "md3-card-sub", "顯示前 50 筆，共 {words.len()} 筆" }
                        }
                        div { class: "preview-table-wrap",
                            table { class: "preview-table",
                                thead {
                                    tr {
                                        th { style: "width:40px", "#" }
                                        th { "正面" }
                                        th { "背面" }
                                    }
                                }
                                tbody {
                                    {words.iter().take(50).enumerate().map(|(i, w)| {
                                        rsx! {
                                            tr {
                                                td { "{i + 1}" }
                                                td { "{w.front}" }
                                                td { "{w.back}" }
                                            }
                                        }
                                    })}
                                    {(words.len() > 50).then(|| {
                                        rsx! { tr { class: "preview-more", td { colspan: "3", "⋯ 還有 {words.len() - 50} 張卡片" } } }
                                    })}
                                }
                            }
                        }
                    }
                    div { class: "destination",
                        div { class: "dest-label",
                            span { class: "material-symbols-outlined", "drive_file_move" }
                            "匯入目標牌組"
                        }
                        // 麵包屑導航
                        div { class: "dest-breadcrumb",
                            button {
                                class: "dest-bc-btn",
                                onclick: move |_| { dest_folder.set(None); selected_deck_id.set(None); show_create_deck.set(false); },
                                span { class: "material-symbols-outlined", "folder" }
                            }
                            {dest_bc.iter().map(|f| {
                                let fid = f.id;
                                let fnm = f.name.clone();
                                rsx! {
                                    span { class: "dest-bc-sep", "›" }
                                    button {
                                        class: "dest-bc-btn",
                                        onclick: move |_| { dest_folder.set(Some(fid)); selected_deck_id.set(None); show_create_deck.set(false); },
                                        "{fnm}"
                                    }
                                }
                            })}
                        }
                        // 項目列表
                        div { class: "dest-items",
                            div { class: "dest-items-scroll",
                            {dest_items_vec.iter().map(|item| {
                                let is_folder = item.is_folder;
                                let id = item.id;
                                let nm = item.name.clone();
                                let wc = item.word_count;
                                let deck_sel = !is_folder && *selected_deck_id.read() == Some(id);
                                rsx! {
                                    if is_folder {
                                        div {
                                            key: "folder-{id}",
                                            class: "dest-folder",
                                            onclick: move |_| { dest_folder.set(Some(id)); selected_deck_id.set(None); show_create_deck.set(false); },
                                            span { class: "material-symbols-outlined dest-folder-icon", "folder" }
                                            div { class: "dest-item-body",
                                                div { class: "dest-item-name", "{nm}" }
                                            }
                                        }
                                    } else {
                                        div {
                                            key: "deck-{id}",
                                            class: format!("dest-deck{}", if deck_sel { " selected" } else { "" }),
                                            onclick: move |_| {
                                                if deck_sel { selected_deck_id.set(None); } else { selected_deck_id.set(Some(id)); }
                                                show_create_deck.set(false);
                                            },
                                            if deck_sel {
                                                span { class: "material-symbols-outlined dest-deck-check", "check" }
                                            }
                                            div { class: "dest-item-body",
                                                div { class: "dest-item-name", "{nm}" }
                                                div { class: "dest-item-meta", "{wc} 詞" }
                                            }
                                        }
                                    }
                                }
                            })}
                            }
                        }
                        // 新增牌組
                        if *show_create_deck.read() {
                            div { class: "dest-create",
                                input {
                                    class: "fsrs-input",
                                    r#type: "text",
                                    placeholder: "新牌組名稱",
                                    value: "{create_deck_name}",
                                    oninput: move |e| create_deck_name.set(e.value()),
                                }
                                button {
                                    class: "text-btn",
                                    onclick: move |_| show_create_deck.set(false),
                                    "✕"
                                }
                            }
                        }
                        if !*show_create_deck.read() {
                            button {
                                class: "text-btn",
                                onclick: move |_| { show_create_deck.set(true); selected_deck_id.set(None); },
                                span { class: "material-symbols-outlined", "add" }
                                "新增牌組"
                            }
                        }
                    }
                    if !error.read().is_empty() {
                        div { class: "error-banner", "{error.read().clone()}" }
                    }
                    div { class: "import-actions",
                        button {
                            class: "md3-btn md3-btn--filled",
                            disabled: loading(),
                            onclick: move |_| do_import(),
                            span { class: "material-symbols-outlined", "file_download" }
                            if loading() { "匯入中…" } else { "匯入到字庫" }
                        }
                    }
                } else if !error.read().is_empty() {
                    div { class: "error-banner", "{error.read().clone()}" }
                }
            }
        }
    }
}

fn main() {
    logging::init();
    dioxus::LaunchBuilder::new()
        .with_cfg(dioxus_desktop::Config::new().with_disable_context_menu(false))
        .launch(App);
}