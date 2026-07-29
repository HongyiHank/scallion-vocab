use crate::model::Word;
use dioxus::document;
use quizlet_scraper::{build_flashcards_url, extract_deck_id, scrape_quizlet_html};
use std::collections::HashSet;
use std::io::Read;

pub fn decode_anki_cell(input: &str) -> String {
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

pub fn parse_anki_text(text: &str) -> Vec<Word> {
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
                Some(Word { front, back, pos: String::new(), pron: String::new(), example: String::new(), synonym: String::new(), antonym: String::new(), tags: Vec::new() })
            }
        })
        .collect()
}

pub fn parse_apkg(data: &[u8]) -> Result<Vec<Word>, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))
        .map_err(|_| "無法解壓縮 .apkg 檔案".to_string())?;
    let mut coll = None;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|_| "讀取 .apkg 失敗".to_string())?;
        if file.name() == "collection.anki2" || file.name().ends_with("collection.anki2") {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(|_| "讀取 collection 失敗".to_string())?;
            coll = Some(buf);
            break;
        }
    }
    let coll = coll.ok_or("找不到 collection.anki2".to_string())?;

    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let tmp = std::env::temp_dir().join(format!("anki_import_{}_{}.anki2", std::process::id(), ts));
    std::fs::write(&tmp, &coll).map_err(|_| "寫入暫存失敗".to_string())?;
    let result = (|| -> Result<Vec<Word>, String> {
        let conn = rusqlite::Connection::open(&tmp).map_err(|_| "讀取牌組資料失敗".to_string())?;
        let mut stmt = conn.prepare("SELECT flds FROM notes").map_err(|_| "讀取卡片失敗".to_string())?;
        let words: Vec<Word> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|_| "解析卡片失敗".to_string())?
            .filter_map(|r| r.ok())
            .filter_map(|flds| {
                let parts: Vec<&str> = flds.split('\x1f').collect();
                if parts.len() < 2 { return None; }
                let front = decode_anki_cell(parts[0]);
                let back = decode_anki_cell(parts[1]);
                if front.is_empty() || back.is_empty() { None }
                else { Some(Word { front, back, pos: String::new(), pron: String::new(), example: String::new(), synonym: String::new(), antonym: String::new(), tags: Vec::new() }) }
            })
            .collect();
        if words.is_empty() { Err("牌組中無有效文字卡片".to_string()) } else { Ok(words) }
    })();
    let _ = std::fs::remove_file(&tmp);
    result
}

// ── Quizlet URL helpers ──────────────────────────────────────────────

pub fn normalize_quizlet_url(raw: &str) -> Option<String> {
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

pub fn parse_quizlet_urls(input: &str) -> Vec<String> {
    let mut seen = HashSet::new();

    input
        .lines()
        .filter_map(normalize_quizlet_url)
        .filter(|url| seen.insert(url.clone()))
        .collect()
}

pub fn clean_recent_urls(urls: Vec<String>, max_len: usize) -> Vec<String> {
    let mut seen = HashSet::new();

    urls.into_iter()
        .filter_map(|url| normalize_quizlet_url(&url))
        .filter(|url| seen.insert(url.clone()))
        .take(max_len)
        .collect()
}

pub async fn save_recent_urls(urls: &[String]) {
    let Ok(json) = serde_json::to_string(urls) else {
        crate::log!("[Prefs::SaveUrls] failed to serialize urls");
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
        crate::log!("[Prefs::SaveUrls] eval failed: {e}");
    }
}

pub async fn fetch_quizlet_multi(urls: &[String]) -> (Vec<Word>, Vec<String>) {
    let mut all_words = Vec::new();
    let mut seen = HashSet::new();
    let mut errors = Vec::new();

    for url in urls {
        crate::log!("[Upload::Fetch] scraping URL: {url}");

        let page_url = match extract_deck_id(url) {
            Ok(deck_id) => build_flashcards_url(&deck_id),
            Err(e) => {
                let msg = format!("{url}: {e}");
                crate::log!("[Upload::Fetch] invalid URL: {e}");
                errors.push(msg);
                continue;
            }
        };

        let cards = match super::fetch_html_via_webview(&page_url).await {
            Ok(html) => {
                crate::log!("[Upload::Fetch] WebView fetch got {} bytes, parsing", html.len());
                match scrape_quizlet_html(&html) {
                    Ok(c) => c,
                    Err(e) => {
                        let msg = format!("{url}: {e}");
                        crate::log!("[Upload::Fetch] HTML parse failed: {e}");
                        errors.push(msg);
                        continue;
                    }
                }
            }
            Err(e) => {
                let msg = format!("{url}: {e}");
                crate::log!("[Upload::Fetch] WebView fetch failed: {e}");
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
                all_words.push(Word { front, back, pos: String::new(), pron: String::new(), example: String::new(), synonym: String::new(), antonym: String::new(), tags: Vec::new() });
                added += 1;
            }
        }
        crate::log!("[Upload::Fetch] {url}: {added}/{raw_count} cards added (total: {})", all_words.len());
    }

    (all_words, errors)
}
