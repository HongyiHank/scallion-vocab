mod logging;
mod css;
mod model;
mod licenses;

use dioxus::document;
use dioxus::prelude::*;
use model::{sleep_ms, FsrsConfig, FsrsRating, QuizState, Screen, Word};
use quizlet_scraper::{build_flashcards_url, extract_deck_id, extract_title, scrape_quizlet_html};
use percent_encoding::percent_decode;
use serde::Deserialize;
use std::collections::HashSet;
use std::time::{Duration, Instant};

const MAX_RECENT_URLS: usize = 5;
const TOAST_DURATION_MS: u64 = 2_800;
const DEFAULT_AUTO_ADVANCE_MS: i64 = 1_000;
const FILE_DIALOG_TIMEOUT_MS: u64 = 60_000;
const ANTI_FOUC_SCRIPT: &str = "try{document.documentElement.setAttribute('data-theme',localStorage.getItem('theme')||'light')}catch(_){}";

#[derive(Clone, Debug, PartialEq)]
struct ToastState {
    id: u64,
    text: String,
}

#[derive(Clone, Copy)]
struct AppSignals {
    screen: Signal<Screen>,
    quiz: Signal<Option<QuizState>>,
    toast: Signal<Option<ToastState>>,
    toast_seq: Signal<u64>,
    is_dark: Signal<bool>,
    infinite_mode: Signal<bool>,
    auto_advance_ms: Signal<i64>,
    fsrs_config: Signal<FsrsConfig>,
    prefs_loaded: Signal<bool>,
    recent_urls: Signal<Vec<String>>,
}

fn push_toast(mut app: AppSignals, msg: impl Into<String>) {
    let id = (*app.toast_seq.read()).wrapping_add(1);
    app.toast_seq.set(id);
    app.toast.set(Some(ToastState {
        id,
        text: msg.into(),
    }));
}

#[allow(non_snake_case)]
fn App() -> Element {
    use_context_provider(|| AppSignals {
        screen: Signal::new(Screen::Upload),
        quiz: Signal::new(None),
        toast: Signal::new(None),
        toast_seq: Signal::new(0),
        is_dark: Signal::new(false),
        infinite_mode: Signal::new(true),
        auto_advance_ms: Signal::new(DEFAULT_AUTO_ADVANCE_MS),
        fsrs_config: Signal::new(FsrsConfig::default()),
        prefs_loaded: Signal::new(false),
        recent_urls: Signal::new(Vec::new()),
    });

    let mut app = use_context::<AppSignals>();

    use_effect(move || {
        spawn(async move {
            load_prefs(app).await;
        });
    });

    // 單一 effect 訂閱所有持久化信號，減少重複 prefs_loaded guard
    use_effect(move || {
        if !*app.prefs_loaded.read() {
            return;
        }

        let is_dark = *app.is_dark.read();
        let infinite = *app.infinite_mode.read();
        let cfg = app.fsrs_config.cloned();
        let ms = *app.auto_advance_ms.read();

        spawn(async move {
            persist_theme(is_dark).await;
            persist_infinite_mode(infinite).await;
            persist_fsrs_config(cfg).await;
            persist_auto_advance_ms(ms).await;
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

    let screen = app.screen.read().clone();
    let toast = app.toast.read().clone();

    rsx! {
        style { "{css::STYLES}" }
        script {
            dangerous_inner_html: ANTI_FOUC_SCRIPT,
        }

        div {
            width: "100%",
            display: "flex",
            justify_content: "center",
            align_items: "center",
            min_height: "100dvh",

            match screen {
                Screen::Upload => rsx! { UploadScreen {} },
                Screen::Quiz => rsx! { QuizScreen {} },
                Screen::QuizFinished => rsx! { QuizFinished {} },
            }
        }

        div {
            class: if toast.is_some() { "toast show" } else { "toast" },
            role: "alert",
            aria_live: "assertive",
            "{toast.as_ref().map(|t| t.text.as_str()).unwrap_or_default()}"
        }
    }
}

#[derive(Debug, Deserialize)]
struct StoredPrefs {
    theme: String,
    urls: Vec<String>,
    infinite_mode: Option<bool>,
    auto_advance_ms: Option<i64>,
    fsrs_config: Option<String>,
}

async fn load_prefs(mut app: AppSignals) {
    let mut eval = document::eval(
        r#"
        try {
            const theme = localStorage.getItem('theme') || 'light';
            document.documentElement.setAttribute('data-theme', theme);
            const urls = JSON.parse(localStorage.getItem('recent_urls') || '[]');
            const infinite_mode = localStorage.getItem('infinite_mode') !== 'false';
            const auto_advance_ms = parseInt(localStorage.getItem('auto_advance_ms'), 10) || null;
            const fsrs_config = localStorage.getItem('fsrs_config') || '';
            dioxus.send(JSON.stringify({ theme, urls: Array.isArray(urls) ? urls : [], infinite_mode, auto_advance_ms, fsrs_config }));
        } catch (_) {
            document.documentElement.setAttribute('data-theme', 'light');
            dioxus.send(JSON.stringify({ theme: 'light', urls: [], infinite_mode: true, auto_advance_ms: null, fsrs_config: '' }));
        }
        "#,
    );

    if let Ok(payload) = eval.recv::<String>().await {
        if let Ok(prefs) = serde_json::from_str::<StoredPrefs>(&payload) {
            app.is_dark.set(prefs.theme == "dark");
            app.infinite_mode.set(prefs.infinite_mode.unwrap_or(true));
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
        } else {
            log!("[Prefs::Load] failed to parse prefs payload");
        }
    } else {
        log!("[Prefs::Load] failed to receive eval result");
    }

    app.prefs_loaded.set(true);
}

async fn persist_theme(is_dark: bool) {
    let theme = if is_dark { "dark" } else { "light" };
    let theme_js = serde_json::to_string(theme).unwrap_or_else(|_| "\"light\"".to_string());

    let script = format!(
        r#"
        try {{
            document.documentElement.setAttribute('data-theme', {theme_js});
            localStorage.setItem('theme', {theme_js});
        }} catch (_) {{
            document.documentElement.setAttribute('data-theme', {theme_js});
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

async fn persist_auto_advance_ms(ms: i64) {
    let js = format!(
        r#"try {{ localStorage.setItem('auto_advance_ms', '{ms}'); }} catch (_) {{}}"#
    );
    if let Err(e) = document::eval(&js).await {
        log!("[Prefs::AutoAdvanceMs] save failed: {e}");
    }
}

fn normalize_quizlet_url(raw: &str) -> Option<String> {
    let raw = raw.trim();

    if raw.is_empty() {
        return None;
    }

    let with_scheme = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_owned()
    } else {
        format!("https://{raw}")
    };

    let parsed = url::Url::parse(&with_scheme).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();

    if host == "quizlet.com" || host.ends_with(".quizlet.com") {
        Some(parsed.to_string())
    } else {
        None
    }
}

fn parse_quizlet_urls(input: &str) -> Vec<String> {
    let mut seen = HashSet::new();

    input
        .lines()
        .filter_map(normalize_quizlet_url)
        .filter(|url| seen.insert(url.clone()))
        .collect()
}

fn clean_recent_urls(urls: Vec<String>, max_len: usize) -> Vec<String> {
    let mut seen = HashSet::new();

    urls.into_iter()
        .filter_map(|url| normalize_quizlet_url(&url))
        .filter(|url| seen.insert(url.clone()))
        .take(max_len)
        .collect()
}

fn format_url_title(url: &str) -> String {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return url.to_string(),
    };

    let segment = parsed
        .path_segments()
        .and_then(|segments| {
            segments
                .filter(|s| !s.is_empty() && !s.contains("quizlet.com"))
                .last()
        })
        .unwrap_or("");

    if segment.is_empty() {
        return url.to_string();
    }

    let decoded = percent_decode(segment.as_bytes())
        .decode_utf8_lossy();

    decoded
        .replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

async fn save_recent_urls(urls: &[String]) {
    let Ok(json) = serde_json::to_string(urls) else {
        log!("[Prefs::SaveUrls] failed to serialize urls");
        return;
    };

    let js_string = serde_json::to_string(&json).unwrap_or_else(|_| "\"[]\"".to_string());

    let script = format!(
        r#"
        try {{
            localStorage.setItem('recent_urls', {js_string});
        }} catch (_) {{}}
        "#
    );

    if let Err(e) = document::eval(&script).await {
        log!("[Prefs::SaveUrls] eval failed: {e}");
    }
}

fn decode_anki_cell(input: &str) -> String {
    input
        .replace("<br />", "\n")
        .replace("<br/>", "\n")
        .replace("<br>", "\n")
        // &amp; must be first so double-encoded entities decode correctly
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .trim()
        .to_string()
}

fn parse_anki_text(text: &str) -> Vec<Word> {
    let clean = text.strip_prefix('\u{FEFF}').unwrap_or(text);

    clean
        .lines()
        .filter_map(|line| {
            let (front, back) = line.split_once('\t')?;
            let front = decode_anki_cell(front);
            let back = decode_anki_cell(back);

            if front.is_empty() || back.is_empty() {
                None
            } else {
                Some(Word { front, back })
            }
        })
        .collect()
}

#[component]
fn UploadScreen() -> Element {
    let mut app = use_context::<AppSignals>();
    let mut url_text = use_signal(String::new);
    let mut fetching = use_signal(|| false);
    let mut fetch_err = use_signal(String::new);
    let mut exporting = use_signal(|| false);
    let mut importing = use_signal(|| false);
    let mut show_html_fallback = use_signal(|| false);
    let mut html_fallback_url = use_signal(String::new);
    let mut html_error = use_signal(String::new);
    let mut html_loading = use_signal(|| false);
    let mut tap_timestamps = use_signal::<Vec<Instant>>(Vec::new);

    let has_urls = url_text.read().lines().any(|l| !l.trim().is_empty());

    let mut show_settings = use_signal(|| false);
    let mut show_licenses = use_signal(|| false);
    let mut selected_dep_name = use_signal(String::new);
    let mut selected_dep_license = use_signal(String::new);
    let mut settings_tab = use_signal(|| 0); // 0 = 一般, 1 = 考試, 2 = FSRS
    let auto_resize_textarea = move || {
        spawn(async move {
            let _ = document::eval(
                r#"let ta=document.querySelector('.url-textarea');if(ta){ta.style.height='auto';ta.style.height=ta.scrollHeight+'px';}"#,
            ).await;
        });
    };

    let export_action = move |_| {
        let urls = parse_quizlet_urls(&url_text.read());
        if urls.is_empty() {
            return;
        }
        exporting.set(true);
        spawn(async move {
            log!("[Upload::Export] starting export for {} urls", urls.len());
            let (all_words, errors) = fetch_quizlet_multi(&urls).await;
            if all_words.is_empty() {
                log!("[Upload::Export] all urls failed: {:?}", errors);
                push_toast(app, "所有網址皆抓取失敗，請檢查網址");
                exporting.set(false);
                return;
            }
            let words_json = serde_json::to_string(&all_words).unwrap_or_default();
            let js = format!(
                r#"
                (function() {{
                    function escapeHtml(text) {{
                        return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
                    }}
                    var words = {words_json};
                    var lines = words.map(function(w) {{
                        var front = escapeHtml(w.front).replace(/\r/g, '').replace(/\t/g, ' ').replace(/\n/g, '<br>');
                        var back  = escapeHtml(w.back ).replace(/\r/g, '').replace(/\t/g, ' ').replace(/\n/g, '<br>');
                        return front + '\t' + back;
                    }});
                    var text = '\uFEFF' + lines.join('\r\n');
                    navigator.clipboard.writeText(text).catch(function() {{}});
                    var blob = new Blob([text], {{ type: 'text/plain;charset=utf-8' }});
                    var url = URL.createObjectURL(blob);
                    var a = document.createElement('a'); a.href = url; a.download = 'anki_cards.txt'; a.click();
                    URL.revokeObjectURL(url);
                }})();
                "#
            );
            if let Err(e) = document::eval(&js).await {
                log!("[Upload::Export] eval failed: {e}");
            }
            let msg = if errors.is_empty() {
                log!("[Upload::Export] success: {} cards from {} urls", all_words.len(), urls.len());
                format!("已導出 {} 個牌組，共 {} 張卡片", urls.len(), all_words.len())
            } else {
                log!("[Upload::Export] partial: {} ok, {} failed", urls.len() - errors.len(), errors.len());
                format!(
                    "已導出 {} 個牌組（{} 個失敗）",
                    urls.len() - errors.len(),
                    errors.len()
                )
            };
            push_toast(app, msg);
            exporting.set(false);
        });
    };

    let import_action = move |_| {
        importing.set(true);
        spawn(async move {
            log!("[Upload::Import] opening file dialog");
            let js = r#"
                let settled = false;
                function send(val) {
                    if (settled) return;
                    settled = true;
                    dioxus.send(val);
                }
                let input = document.createElement('input');
                input.type = 'file'; input.accept = '.txt,.csv';
                input.onchange = async (e) => {
                    try {
                        let file = e.target.files[0];
                        if (!file) { send(""); return; }
                        let text = await file.text();
                        send(text);
                    } catch(err) {
                        send("");
                    }
                };
                input.addEventListener('cancel', () => send(""));
                input.click();
                setTimeout(() => send(""), __TIMEOUT_MS__);
            "#
            .replace("__TIMEOUT_MS__", &FILE_DIALOG_TIMEOUT_MS.to_string());
            let mut eval = document::eval(&js);
            match eval.recv::<serde_json::Value>().await {
                Ok(val) => {
                    if let Some(text) = val.as_str() {
                        if text.is_empty() {
                            log!("[Upload::Import] cancelled or empty");
                            push_toast(app, "已取消匯入");
                        } else {
                            log!("[Upload::Import] received {} bytes", text.len());
                            let words = parse_anki_text(text);
                            if !words.is_empty() {
                                log!("[Upload::Import] success: {} words", words.len());
                                let mut qs = QuizState::new(words, *app.infinite_mode.read(), app.fsrs_config.cloned());
                                if !qs.gen_question() {
                                    push_toast(app, "無法產生題目（無有效單字）");
                                } else {
                                    app.quiz.set(Some(qs));
                                    app.screen.set(Screen::Quiz);
                                    push_toast(app, "成功匯入 Anki 檔案！");
                                }
                            } else {
                                log!("[Upload::Import] no valid cards found");
                                push_toast(app, "檔案格式錯誤或無有效卡片");
                            }
                        }
                    } else {
                        log!("[Upload::Import] unexpected value type");
                    }
                }
                Err(e) => log!("[Upload::Import] recv failed: {e}"),
            }
            importing.set(false);
        });
    };

    let open_fallback_page = move |_| {
        let url = html_fallback_url.read().trim().to_string();
        let Some(url) = normalize_quizlet_url(&url) else { return };
        spawn(async move {
            let url_js = serde_json::to_string(&url).unwrap_or_else(|_| "\"\"".to_string());
            let js = format!(
                r#"(function() {{
                    if (window.AndroidExternal && typeof window.AndroidExternal.openUrl === 'function') {{
                        window.AndroidExternal.openUrl({url_js});
                    }} else {{
                        window.open({url_js}, '_blank', 'noopener,noreferrer');
                    }}
                }})();"#
            );
            if let Err(e) = document::eval(&js).await {
                log!("[Upload::HtmlFallback] open_url eval failed: {e}");
            }
        });
    };

    let html_import_action = move |_| {
        log!("[Upload::HtmlFallback] button clicked");
        spawn(async move {
            // Android WebView may skip oninput on paste, so always read from the DOM.
            let mut eval = document::eval(
                r#"(function(){var ta=document.querySelector('.html-textarea');dioxus.send(ta?ta.value:'');})()"#,
            );
            let html = eval.recv::<String>().await.unwrap_or_default();
            if html.trim().is_empty() {
                html_error.set("請貼上 HTML 內容".to_string());
                return;
            }
            html_loading.set(true);
            html_error.set(String::new());
            let url = html_fallback_url.read().clone();
            log!("[Upload::HtmlFallback] starting import: {} bytes", html.len());
            match scrape_words_from_html(&html) {
                Ok((words, title)) => {
                    log!("[Upload::HtmlFallback] success: {} words, title='{}'", words.len(), title);
                    let mut qs = QuizState::new(words, *app.infinite_mode.read(), app.fsrs_config.cloned());
                    if !qs.gen_question() {
                        html_error.set("無法產生題目（無有效單字）".to_string());
                        html_loading.set(false);
                        return;
                    }
                    app.quiz.set(Some(qs));
                    app.screen.set(Screen::Quiz);
                    if let Some(u) = normalize_quizlet_url(&url) {
                        let mut recent = app.recent_urls.cloned();
                        recent.retain(|x| x != &u);
                        recent.insert(0, u.clone());
                        recent.truncate(MAX_RECENT_URLS);
                        app.recent_urls.set(recent.clone());
                        save_recent_urls(&recent).await;
                    }
                    show_html_fallback.set(false);
                    html_error.set(String::new());
                    push_toast(app, format!("成功從 HTML 匯入「{title}」！"));
                }
                Err(e) => {
                    log!("[Upload::HtmlFallback] error: {e}");
                    html_error.set(e);
                }
            }
            html_loading.set(false);
        });
    };

    let cancel_html_fallback = move |_| {
        show_html_fallback.set(false);
        html_loading.set(false);
        html_error.set(String::new());
    };

    let recent_urls = app.recent_urls.cloned();

    let show_detail = !selected_dep_name.read().is_empty();
    let detail_name = match show_detail {
        true => selected_dep_name.read().clone(),
        false => String::new(),
    };
    let detail_text = match show_detail {
        true => licenses::get_license_text(&selected_dep_license.read()).to_owned(),
        false => String::new(),
    };

    rsx! {
        button {
            class: "top-icon-btn",
            style: "left: 20px;",
            onclick: move |_| show_settings.set(true),
            span { class: "material-symbols-outlined", "settings" }
        }
        div { class: "upload-wrapper",
            section { class: "upload-container",
                h2 { "青蔥背單字" }
                p { class: "subtitle", "輸入 " strong { "Quizlet" } " 網址即可開始測驗" }
                textarea {
                    class: "url-textarea",
                    aria_label: "Quizlet URLs",
                    value: "{url_text}",
                    placeholder: "每行一個 Quizlet 網址，例如：\nhttps://quizlet.com/123/deck/\nhttps://quizlet.com/456/flash-cards/",
                    disabled: fetching(),
                    rows: "4",
                    oninput: move |e| {
                        url_text.set(e.value());
                        auto_resize_textarea();
                    },
                }
                div { class: "action-btns-row",
                    button {
                        class: format!("go-btn{}", if fetching() || !has_urls { " inert" } else { "" }),
                        onclick: move |_| {
                            let urls = parse_quizlet_urls(&url_text.read());
                            if urls.is_empty() {
                                let now = Instant::now();
                                let mut taps = tap_timestamps.write();
                                taps.push(now);
                                taps.retain(|t| now.duration_since(*t).as_secs_f64() < 1.0);
                                if taps.len() >= 5 {
                                    taps.clear();
                                    drop(taps);
                                    log!("[Upload::EasterEgg] 5 rapid taps detected, forcing HTML fallback");
                                    html_error.set(String::new());
                                    show_html_fallback.set(true);
                                }
                                return;
                            }
                            fetching.set(true);
                            fetch_err.set(String::new());
                            show_html_fallback.set(false);
                            html_error.set(String::new());
                            spawn(async move {
                                log!("[Upload::Fetch] fetching {} urls", urls.len());
                                let (all_words, errors) = fetch_quizlet_multi(&urls).await;
                                if all_words.is_empty() {
                                    log!("[Upload::Fetch] all urls failed, showing HTML fallback");
                                    fetch_err.set(errors.join("\n"));
                                    if let Some(first_url) = urls.first() {
                                        html_fallback_url.set(first_url.clone());
                                    }
                                    show_html_fallback.set(true);
                                    fetching.set(false);
                                    return;
                                }

                                let word_count = all_words.len();
                                let mut qs = QuizState::new(all_words, *app.infinite_mode.read(), app.fsrs_config.cloned());
                                if !qs.gen_question() {
                                    push_toast(app, "無法產生題目（無有效單字）");
                                    fetching.set(false);
                                    return;
                                }
                                app.quiz.set(Some(qs));
                                app.screen.set(Screen::Quiz);
                                let mut recent = app.recent_urls.cloned();
                                for u in &urls {
                                    recent.retain(|x| x != u);
                                    recent.insert(0, u.clone());
                                }
                                recent.truncate(MAX_RECENT_URLS);
                                app.recent_urls.set(recent.clone());
                                if errors.is_empty() {
                                    push_toast(app, format!("成功載入 {} 個單字！", word_count));
                                } else {
                                    push_toast(app, format!("成功載入！({} 個網址失敗)", errors.len()));
                                }
                                save_recent_urls(&recent).await;
                                fetching.set(false);
                            });
                        },
                        if fetching() { "抓取中…" } else { "匯入牌組並開始測驗" }
                    }
                    button {
                        class: "text-btn",
                        disabled: exporting() || !has_urls,
                        onclick: export_action,
                        span { class: "material-symbols-outlined", "upload_file" } if exporting() { " 導出中…" } else { " 導出 Anki" }
                    }
                    button {
                        class: "text-btn",
                        disabled: importing(),
                        onclick: import_action,
                        span { class: "material-symbols-outlined", "download" } if importing() { " 匯入中…" } else { " 匯入 Anki 檔案" }
                    }
                }
                if !fetch_err.read().is_empty() {
                    div { class: "fetch-error", "{fetch_err.read().clone()}" }
                }
                if *show_html_fallback.read() {
                    div { class: "html-fallback",
                        p { class: "fallback-title", "抓取失敗，是否手動貼上網頁 HTML？" }
                        p { class: "fallback-hint",
                            "1. 點擊「開啟網頁」在瀏覽器開啟 Quizlet\n2. 在網址列最前面加入 view-source: 再前往\n3. 等待載入後，全選並複製全部 HTML\n4. 回到本 App，貼到下方文字框\n5. 點擊「用 HTML 匯入」"
                        }
                        input {
                            class: "fallback-url-input",
                            r#type: "url",
                            aria_label: "Quizlet 網址",
                            value: "{html_fallback_url}",
                            placeholder: "https://quizlet.com/123/flash-cards/",
                            oninput: move |e| { html_fallback_url.set(e.value()); },
                        }
                        button {
                            class: "open-page-btn",
                            disabled: html_fallback_url.read().trim().is_empty(),
                            onclick: open_fallback_page,
                            "開啟網頁 " span { class: "material-symbols-outlined", "open_in_new" }
                        }
                        textarea {
                            class: "html-textarea",
                            aria_label: "網頁 HTML",
                            placeholder: "在此貼上 Quizlet 網頁的完整 HTML…",
                        }
                        if !html_error.read().is_empty() {
                            div { class: "fetch-error", "{html_error.read().clone()}" }
                        }
                        div { class: "fallback-actions",
                            button {
                                class: format!("go-btn{}", if html_loading() { " inert" } else { "" }),
                                onclick: html_import_action,
                                if html_loading() { "解析中…" } else { "用 HTML 匯入" }
                            }
                            button {
                                class: "text-btn",
                                disabled: html_loading(),
                                onclick: cancel_html_fallback,
                                "取消"
                            }
                        }
                    }
                }
            }

            if !recent_urls.is_empty() {
                section { class: "history-card",
                    h3 { "最近測驗紀錄" }
                    div { class: "history-list",
                        {recent_urls.into_iter().map(|u| {
                            let title = format_url_title(&u);
                            let u_clone = u.clone();
                            rsx! {
                                button {
                                    key: "{u}",
                                    class: "history-item",
                                    title: "{u}",
                                    onclick: move |_| {
                                        let current = url_text.read().trim().to_owned();
                                        if current.is_empty() {
                                            url_text.set(u_clone.clone());
                                        } else if !current.contains(&u_clone) {
                                            url_text.set(format!("{}\n{}", current, u_clone));
                                        }
                                        auto_resize_textarea();
                                    },
                                    span { class: "hist-icon material-symbols-outlined", "link" }
                                    span { class: "hist-text", "{title}" }
                                    span { class: "hist-arrow material-symbols-outlined", "chevron_right" }
                                }
                            }
                        })}
                    }
                }
            }
        }
        if *show_settings.read() {
            div { class: "settings-overlay",
                div { class: "settings-topbar",
                    button {
                        class: "settings-close",
                        onclick: move |_| show_settings.set(false),
                        span { class: "material-symbols-outlined", "arrow_back" }
                    }
                    span { class: "settings-topbar-title", "設定" }
                    button {
                        class: "settings-topbar-btn",
                        title: "還原預設值",
                        onclick: move |_| {
                            app.fsrs_config.set(FsrsConfig::default());
                            app.is_dark.set(false);
                            app.infinite_mode.set(true);
                            push_toast(app, "已還原預設值");
                        },
                        span { class: "material-symbols-outlined", "restart_alt" }
                    }
                    button {
                        class: "settings-topbar-btn",
                        title: "開源許可證",
                        onclick: move |_| show_licenses.set(true),
                        span { class: "material-symbols-outlined", "description" }
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
                        div {
                            class: "settings-item",
                            onclick: move |_| {
                                let new_val = !*app.is_dark.read();
                                app.is_dark.set(new_val);
                            },
                            div { class: "settings-item-icon",
                                span { class: "material-symbols-outlined",
                                    if *app.is_dark.read() { "dark_mode" } else { "light_mode" }
                                }
                            }
                            div { class: "settings-item-label", "深色主題" }
                            div {
                                class: if *app.is_dark.read() { "settings-switch on" } else { "settings-switch" },
                            }
                        }
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
                    }
                } else if *settings_tab.read() == 1 {
                    div { class: "settings-body",
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
                div { class: "settings-bottom",
                    button {
                        class: "settings-github-icon",
                        title: "GitHub 原始碼",
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
                    span { class: "settings-version", "v1.1" }
                }
            }
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
                                        selected_dep_name.set(n.clone());
                                        selected_dep_license.set(lf.clone());
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
                                    selected_dep_name.set(String::new());
                                    selected_dep_license.set(String::new());
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

type FetchResult<T> = Result<T, String>;

async fn fetch_html_via_webview(url: &str) -> FetchResult<String> {
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

async fn fetch_quizlet_multi(urls: &[String]) -> (Vec<Word>, Vec<String>) {
    let mut all_words = Vec::new();
    let mut seen = HashSet::new();
    let mut errors = Vec::new();

    for url in urls {
        log!("[Upload::Fetch] scraping URL: {url}");

        let page_url = match extract_deck_id(url) {
            Ok(deck_id) => build_flashcards_url(&deck_id),
            Err(e) => {
                let msg = format!("{url}: {e}");
                log!("[Upload::Fetch] invalid URL: {e}");
                errors.push(msg);
                continue;
            }
        };

        let cards = match fetch_html_via_webview(&page_url).await {
            Ok(html) => {
                log!("[Upload::Fetch] WebView fetch got {} bytes, parsing", html.len());
                match scrape_quizlet_html(&html) {
                    Ok(c) => c,
                    Err(e) => {
                        let msg = format!("{url}: {e}");
                        log!("[Upload::Fetch] HTML parse failed: {e}");
                        errors.push(msg);
                        continue;
                    }
                }
            }
            Err(e) => {
                let msg = format!("{url}: {e}");
                log!("[Upload::Fetch] WebView fetch failed: {e}");
                errors.push(msg);
                continue;
            }
        };

        let raw_count = cards.len();
        let mut added = 0;
        for card in cards {
            let front = card.term.trim().to_string();
            let back = card.definition.trim().to_string();
            if front.is_empty() || back.is_empty() {
                continue;
            }
            if seen.insert((front.clone(), back.clone())) {
                all_words.push(Word { front, back });
                added += 1;
            }
        }
        log!("[Upload::Fetch] {url}: {added}/{raw_count} cards added (total: {})", all_words.len());
    }

    (all_words, errors)
}

fn scrape_words_from_html(html: &str) -> FetchResult<(Vec<Word>, String)> {
    let cards = scrape_quizlet_html(html).map_err(|e| format!("{e}"))?;
    let title = extract_title(html);

    // 與 fetch_quizlet_multi 一致：trim 並過濾掉 front/back 為空的卡片（圖片卡等）
    let mut seen = HashSet::new();
    let words: Vec<Word> = cards
        .into_iter()
        .filter_map(|card| {
            let front = card.term.trim().to_string();
            let back = card.definition.trim().to_string();
            if front.is_empty() || back.is_empty() {
                return None;
            }
            if seen.insert((front.clone(), back.clone())) {
                Some(Word { front, back })
            } else {
                None
            }
        })
        .collect();

    if words.is_empty() {
        return Err("HTML 中找不到有效文字卡片（可能為圖片卡或無文字內容）".into());
    }
    Ok((words, title))
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
        if app.fsrs_config.read().enabled {
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
                            onclick: move |_| { app.quiz.set(None); app.screen.set(Screen::Upload); },
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

    // 只複製目標單字的 front/back（2 個 String），不複製整個 QuizState
    let (front, back, answered, ask_front) = {
        let qs = app.quiz.read();
        let Some(qs) = qs.as_ref() else {
            return rsx! { div {} };
        };
        let Some(q) = qs.current_question() else {
            return rsx! { div {} };
        };
        let word = &qs.words[q.target_idx];
        (word.front.clone(), word.back.clone(), q.answered, q.ask_front)
    };

    if answered {
        rsx! {
            h2 { id: "question-word",
                span { class: "ans-en", "{front}" }
                span { class: "ans-zh", "{back}" }
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
                min: "1",
                value: "{value}",
                placeholder: "毫秒",
                oninput: move |e| {
                    let v = e.value().trim().to_string();
                    if v.is_empty() {
                        err.set(String::new());
                        return;
                    }
                    match v.parse::<u64>() {
                        Ok(n) if n > 0 => {
                            let mut c = app.fsrs_config.cloned();
                            match field.as_str() {
                                "good" if n <= c.easy_threshold_ms => {
                                    err.set("值必須大於 簡單".to_string());
                                    return;
                                }
                                "hard" if n <= c.good_threshold_ms => {
                                    err.set("值必須大於 良好".to_string());
                                    return;
                                }
                                "hard" => c.hard_threshold_ms = n,
                                "good" => c.good_threshold_ms = n,
                                "easy" => c.easy_threshold_ms = n,
                                _ => {}
                            }
                            err.set(String::new());
                            app.fsrs_config.set(c);
                        }
                        _ => {
                            err.set("請輸入正數".to_string());
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

    let (correct, wrong) = {
        let qs = app.quiz.read();
        let qs = match qs.as_ref() {
            Some(qs) => qs,
            None => return rsx! { div {} },
        };
        let ok = qs.history.iter().filter(|h| h.answered && !h.skipped && h.selected_idx == Some(h.correct_opt)).count();
        let ko = qs.history.iter().filter(|h| h.answered && !h.skipped && h.selected_idx != Some(h.correct_opt)).count();
        (ok, ko)
    };

    rsx! {
        div { class: "finish-screen",
            div { class: "finish-icon",
                span { class: "material-symbols-outlined", "check_circle" }
            }
            div { class: "finish-title", "測驗完成" }
            div { class: "finish-score",
                span { class: "correct", "{correct}" } " 題正確　"
                span { class: "wrong", "{wrong}" } " 題錯誤"
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
                    app.quiz.set(None);
                    app.screen.set(Screen::Upload);
                },
                "返回主頁"
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