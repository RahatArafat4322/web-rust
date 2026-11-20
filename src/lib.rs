use js_sys::{Date, Math};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    CanvasRenderingContext2d, Document, Element, Event, Headers, HtmlCanvasElement,
    HtmlIFrameElement, HtmlImageElement, HtmlInputElement, KeyboardEvent, MouseEvent, Request,
    RequestCredentials, RequestInit, RequestMode, Response, TouchEvent, Window,
};

const CHANNEL_NAME: &str = "Nextcircuit";
const LOGO_URL: &str = "image_499882.png";
const CHANNEL_ID: &str = "UCYjqxJnWhLHKPc9mkuc2Mcg";
const API_KEY: &str = "AIzaSyAAycUXr6jdqRUgQUym-Czq63BqDG0SvXg";
const STORAGE_KEY: &str = "nextcircuit_v3_config";

const DEFAULT_LOGO_SVG: &str = r##"<svg width="36" height="36" viewBox="0 0 36 36" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="9" y="9" width="18" height="18" rx="4" stroke="#ffffff" stroke-width="2.2"/><path d="M14 22V14L22 22V14" stroke="#ffffff" stroke-width="2.2" stroke-linecap="round"/><path d="M13 4V9M23 4V9M13 27V32M23 27V32M4 13H9M4 23H9M27 13H32M27 23H32" stroke="#ffffff" stroke-width="2" stroke-linecap="round"/></svg>"##;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LinkItem {
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    drive_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct SoftwareItem {
    #[serde(default)]
    name: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    size: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    stars: Option<u64>,
    #[serde(default)]
    credit: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    drive_url: Option<String>,
    #[serde(default)]
    links: Option<Vec<LinkItem>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct SoftwareData {
    #[serde(default)]
    github: Vec<SoftwareItem>,
    #[serde(default)]
    windows: Vec<SoftwareItem>,
    #[serde(default)]
    linux: Vec<SoftwareItem>,
    #[serde(default)]
    android: Vec<SoftwareItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct VideoItem {
    id: String,
    title: String,
    views: u64,
    published_at: String,
    thumbnail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct TopicItem {
    id: String,
    label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SiteConfig {
    #[serde(default = "default_channel_name")]
    channel_name: String,
    #[serde(default = "default_handle")]
    handle: String,
    #[serde(default = "default_bio")]
    bio: String,
    #[serde(default = "default_phrases")]
    typewriter_phrases: Vec<String>,
    #[serde(default = "default_topics")]
    topics: Vec<TopicItem>,
    #[serde(default = "default_channel_id")]
    channel_id: String,
    #[serde(default = "default_api_key")]
    api_key: String,
}

fn default_channel_name() -> String { CHANNEL_NAME.to_string() }
fn default_handle() -> String { "@next_circuit".to_string() }
fn default_bio() -> String { "Embedded systems, STM32, ESP32, Arduino tutorials & hardware engineering projects.".to_string() }
fn default_phrases() -> Vec<String> {
    vec![
        "Embedded Systems & PCB Design".to_string(),
        "STM32 & ARM Cortex-M Firmware".to_string(),
        "ESP32 & IoT Wireless Projects".to_string(),
        "Arduino & Hardware Engineering".to_string(),
        "Open-Source Tools & Software".to_string(),
    ]
}
fn default_topics() -> Vec<TopicItem> {
    vec![
        TopicItem { id: "all".into(), label: "All".into() },
        TopicItem { id: "stm32".into(), label: "STM32".into() },
        TopicItem { id: "esp32".into(), label: "ESP32".into() },
        TopicItem { id: "arduino".into(), label: "Arduino".into() },
        TopicItem { id: "pcb".into(), label: "PCB".into() },
    ]
}
fn default_channel_id() -> String { CHANNEL_ID.to_string() }
fn default_api_key() -> String { API_KEY.to_string() }

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            channel_name: default_channel_name(),
            handle: default_handle(),
            bio: default_bio(),
            typewriter_phrases: default_phrases(),
            topics: default_topics(),
            channel_id: default_channel_id(),
            api_key: default_api_key(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct StoredConfig {
    #[serde(default)]
    software: SoftwareData,
    #[serde(default)]
    videos: Vec<VideoItem>,
}

struct AppState {
    config: StoredConfig,
    site: SiteConfig,
    search_queries: HashMap<String, String>,
    active_topic: String,
    software_sync_error: Option<String>,
    youtube_sync_error: Option<String>,
    toast_timer: Option<i32>,
}

impl AppState {
    fn new() -> Self {
        let mut search_queries = HashMap::with_capacity(5);
        for tab in ["youtube", "github", "windows", "linux", "android"] {
            search_queries.insert(tab.to_string(), String::new());
        }
        Self {
            config: load_safe_config(),
            site: SiteConfig::default(),
            search_queries,
            active_topic: "all".to_string(),
            software_sync_error: None,
            youtube_sync_error: None,
            toast_timer: None,
        }
    }
}

#[inline]
fn window() -> Window {
    web_sys::window().expect("no global window")
}

#[inline]
fn document() -> Document {
    window().document().expect("no document on window")
}

#[inline]
fn is_valid_tab(tab: &str) -> bool {
    matches!(tab, "youtube" | "github" | "windows" | "linux" | "android")
}

#[inline]
fn is_valid_video_id(id: &str) -> bool {
    id.len() == 11
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn sanitize_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return "#".to_string();
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("//")
        || lower.starts_with("javascript:")
        || lower.starts_with("vbscript:")
        || lower.starts_with("data:")
        || lower.starts_with("file:")
        || lower.starts_with("blob:")
    {
        return "#".to_string();
    }

    if !trimmed.contains(':')
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-'))
    {
        return trimmed.to_string();
    }

    if lower.starts_with("https://") {
        return trimmed.to_string();
    }

    "#".to_string()
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '/' => out.push_str("&#x2F;"),
            '`' => out.push_str("&#x60;"),
            '=' => out.push_str("&#x3D;"),
            _ => out.push(ch),
        }
    }
    out
}

#[inline]
fn encode_uri_component(s: &str) -> String {
    js_sys::encode_uri_component(s)
        .as_string()
        .unwrap_or_default()
}

fn sanitize_software_list(list: Vec<SoftwareItem>) -> Vec<SoftwareItem> {
    list.into_iter()
        .filter(|item| !item.name.trim().is_empty())
        .map(|mut item| {
            item.name = item.name.chars().take(120).collect();
            item.version = item.version.map(|v| v.chars().take(32).collect());
            item.size = item.size.map(|s| s.chars().take(32).collect());
            item.description = item.description.map(|d| d.chars().take(220).collect());
            item.language = item.language.map(|l| l.chars().take(40).collect());
            item.credit = item.credit.map(|c| c.chars().take(60).collect());
            item.url = item.url.map(|u| sanitize_url(&u));
            item.drive_url = item.drive_url.map(|u| sanitize_url(&u));
            item.links = item.links.map(|links| {
                links
                    .into_iter()
                    .map(|mut l| {
                        l.label = Some(
                            l.label
                                .unwrap_or_else(|| "Secure Download".to_string())
                                .chars()
                                .take(60)
                                .collect(),
                        );
                        let raw = l
                            .url
                            .as_deref()
                            .or(l.drive_url.as_deref())
                            .unwrap_or("#");
                        l.url = Some(sanitize_url(raw));
                        l
                    })
                    .collect()
            });
            item
        })
        .collect()
}

fn load_safe_config() -> StoredConfig {
    let mut base = StoredConfig::default();
    if let Ok(Some(storage)) = window().local_storage() {
        if let Ok(Some(raw)) = storage.get_item(STORAGE_KEY) {
            if let Ok(parsed) = serde_json::from_str::<StoredConfig>(&raw) {
                base.software = SoftwareData {
                    github: sanitize_software_list(parsed.software.github),
                    windows: sanitize_software_list(parsed.software.windows),
                    linux: sanitize_software_list(parsed.software.linux),
                    android: sanitize_software_list(parsed.software.android),
                };
                base.videos = parsed
                    .videos
                    .into_iter()
                    .filter(|v| is_valid_video_id(&v.id))
                    .map(|mut v| {
                        v.title = v.title.chars().take(200).collect();
                        v.thumbnail = sanitize_url(&v.thumbnail);
                        v
                    })
                    .collect();
            }
        }
    }
    base
}

fn save_config(config: &StoredConfig) {
    if let Ok(Some(storage)) = window().local_storage() {
        if let Ok(json) = serde_json::to_string(config) {
            let _ = storage.set_item(STORAGE_KEY, &json);
        }
    }
}

fn format_views(num: u64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M views", (num as f64) / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K views", (num as f64) / 1_000.0)
    } else {
        format!("{} views", num)
    }
}

fn format_short_date(iso: &str) -> String {
    if iso.len() >= 10 {
        iso[..10].to_string()
    } else {
        "Unknown Date".to_string()
    }
}

fn update_count_badge(tab_id: &str, count: usize) {
    if let Some(badge) = document().get_element_by_id(&format!("count-{}", tab_id)) {
        badge.set_text_content(Some(&count.to_string()));
    }
}

fn animate_section_heading(tab_id: &str) {
    let Some(tab_el) = document().get_element_by_id(&format!("tab-{}", tab_id)) else {
        return;
    };
    let Ok(Some(h1)) = tab_el.query_selector("h1.animated-title") else {
        return;
    };
    let raw_text = h1
        .get_attribute("data-text")
        .unwrap_or_else(|| h1.text_content().unwrap_or_default().trim().to_string());

    let mut html = String::with_capacity(raw_text.len() * 64);
    for (idx, ch) in raw_text.chars().enumerate() {
        if ch == ' ' {
            html.push(' ');
        } else {
            let delay = (idx as f64) * 0.025;
            html.push_str(&format!(
                r#"<span class="char-reveal" style="animation-delay: {:.3}s;">{}</span>"#,
                delay,
                escape_html(&ch.to_string())
            ));
        }
    }
    h1.set_inner_html(&html);
}

fn get_primary_url(item: &SoftwareItem) -> String {
    if let Some(links) = &item.links {
        if let Some(first) = links.first() {
            let u = first
                .url
                .as_deref()
                .or(first.drive_url.as_deref())
                .unwrap_or("#");
            return sanitize_url(u);
        }
    }
    if let Some(drive) = &item.drive_url {
        let s = sanitize_url(drive);
        if s != "#" {
            return s;
        }
    }
    if let Some(url) = &item.url {
        return sanitize_url(url);
    }
    "#".to_string()
}

fn build_download_buttons(item: &SoftwareItem) -> String {
    if let Some(links) = &item.links {
        if !links.is_empty() {
            let mut html = String::with_capacity(links.len() * 260 + 32);
            html.push_str(r#"<div class="links-stack">"#);
            for link in links {
                let raw = link
                    .url
                    .as_deref()
                    .or(link.drive_url.as_deref())
                    .unwrap_or("#");
                let href = sanitize_url(raw);
                let label = escape_html(link.label.as_deref().unwrap_or("Secure Download"));
                html.push_str(&format!(
                    r#"<a href="{}" target="_blank" rel="noopener noreferrer nofollow" class="drive-btn"><svg style="width:16px; margin-right:8px;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>{}</a>"#,
                    escape_html(&href),
                    label
                ));
            }
            html.push_str("</div>");
            return html;
        }
    }

    let href = get_primary_url(item);
    format!(
        r#"<div class="links-stack"><a href="{}" target="_blank" rel="noopener noreferrer nofollow" class="drive-btn"><svg style="width:16px; margin-right:8px;" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>Secure Download</a></div>"#,
        escape_html(&href)
    )
}

fn show_toast(state: &Rc<RefCell<AppState>>, message: &str) {
    let Some(toast) = document().get_element_by_id("toastBanner") else {
        return;
    };
    toast.set_text_content(Some(message));
    let _ = toast.class_list().add_1("show");

    if let Some(prev) = state.borrow_mut().toast_timer.take() {
        window().clear_timeout_with_handle(prev);
    }

    let cb = Closure::once(move || {
        if let Some(t) = document().get_element_by_id("toastBanner") {
            let _ = t.class_list().remove_1("show");
        }
    });
    if let Ok(id) = window().set_timeout_with_callback_and_timeout_and_arguments_0(
        cb.as_ref().unchecked_ref(),
        2600,
    ) {
        state.borrow_mut().toast_timer = Some(id);
    }
    cb.forget();
}

fn render_videos(state: &Rc<RefCell<AppState>>) {
    let st = state.borrow();
    let sort_mode = document()
        .get_element_by_id("sortMode")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .unwrap_or_else(|| "views".to_string());

    let query = st
        .search_queries
        .get("youtube")
        .map(String::as_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let topic = st.active_topic.as_str();

    let mut sorted: Vec<&VideoItem> = st
        .config
        .videos
        .iter()
        .filter(|v| {
            let title_lower = v.title.to_ascii_lowercase();
            let matches_query = query.is_empty() || title_lower.contains(&query);
            let matches_topic = topic == "all" || title_lower.contains(topic);
            matches_query && matches_topic
        })
        .collect();

    if sort_mode == "views" {
        sorted.sort_unstable_by(|a, b| b.views.cmp(&a.views));
    } else {
        sorted.sort_unstable_by(|a, b| b.published_at.cmp(&a.published_at));
    }

    update_count_badge("youtube", sorted.len());

    let Some(grid) = document().get_element_by_id("videoGrid") else {
        return;
    };

    if !st.config.videos.is_empty() && sorted.is_empty() {
        grid.set_inner_html(
            r#"<div style="grid-column: 1 / -1; padding: 40px; text-align: center; color: var(--text-secondary);">No matching videos found.</div>"#,
        );
        return;
    }

    if sorted.is_empty() {
        if let Some(err) = &st.youtube_sync_error {
            grid.set_inner_html(&format!(
                r#"<div class="error-banner">{}</div>"#,
                escape_html(err)
            ));
        } else {
            grid.set_inner_html(
                r#"<div style="grid-column: 1 / -1; padding: 60px 32px; background: rgba(0,0,0,0.35); border: 1px dashed var(--border-subtle); border-radius: 12px; text-align: center;"><h3 style="font-size: 16px; font-weight: 500; color: var(--text-primary); margin-bottom: 8px;">Waiting for Feed Sync</h3><p style="font-size: 13px; color: var(--text-secondary);">Videos will appear here securely once synced.</p></div>"#,
            );
        }
        return;
    }

    let mut html = String::with_capacity(sorted.len() * 420);
    for (idx, v) in sorted.iter().enumerate() {
        let delay = ((idx as f64) * 0.02).min(0.35);
        let date_str = format_short_date(&v.published_at);
        html.push_str(&format!(
            r#"<article class="video-card" data-video-id="{}" data-video-title="{}" style="animation-delay: {:.3}s;" tabindex="0" role="button"><div class="thumb-container"><img src="{}" alt="{}" loading="lazy" /><div class="play-overlay"><div class="play-circle">▶</div></div></div><div class="card-body"><h4 class="video-title">{}</h4><div class="video-stats"><span class="views-pill">{}</span><span><time datetime="{}">{}</time></span></div></div></article>"#,
            escape_html(&v.id),
            escape_html(&v.title),
            delay,
            escape_html(&sanitize_url(&v.thumbnail)),
            escape_html(&v.title),
            escape_html(&v.title),
            format_views(v.views),
            escape_html(&v.published_at),
            escape_html(&date_str),
        ));
    }

    grid.set_inner_html(&html);
}

fn render_github_repos(state: &Rc<RefCell<AppState>>) {
    let st = state.borrow();
    let Some(grid) = document().get_element_by_id("githubGrid") else {
        return;
    };

    if let Some(err) = &st.software_sync_error {
        grid.set_inner_html(&format!(
            r#"<div class="error-banner">{}</div>"#,
            escape_html(err)
        ));
        update_count_badge("github", 0);
        return;
    }

    let all_repos = &st.config.software.github;
    let query_raw = st
        .search_queries
        .get("github")
        .map(String::as_str)
        .unwrap_or("");
    let query = query_raw.to_ascii_lowercase();

    let repos: Vec<&SoftwareItem> = all_repos
        .iter()
        .filter(|r| {
            query.is_empty()
                || r.name.to_ascii_lowercase().contains(&query)
                || r.description
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains(&query)
                || r.language
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains(&query)
        })
        .collect();

    update_count_badge("github", repos.len());

    if all_repos.is_empty() {
        grid.set_inner_html(
            r#"<div style="grid-column: 1/-1; padding: 40px; text-align: center; color: var(--text-secondary);">No repositories linked yet. Edit software.json to add some!</div>"#,
        );
        return;
    }

    if repos.is_empty() {
        grid.set_inner_html(&format!(
            r#"<div style="grid-column: 1/-1; padding: 40px; text-align: center; color: var(--text-secondary);">No repositories matching "{}"</div>"#,
            escape_html(query_raw)
        ));
        return;
    }

    let mut html = String::with_capacity(repos.len() * 640);
    for (idx, repo) in repos.iter().enumerate() {
        let href = sanitize_url(
            repo.url
                .as_deref()
                .or(repo.drive_url.as_deref())
                .unwrap_or("#"),
        );
        let delay = ((idx as f64) * 0.02).min(0.35);

        let mut pills = String::new();
        if let Some(lang) = &repo.language {
            if !lang.is_empty() {
                pills.push_str(&format!(
                    r#"<span class="meta-pill accent">{}</span>"#,
                    escape_html(lang)
                ));
            }
        }
        if let Some(stars) = repo.stars {
            if stars > 0 {
                pills.push_str(&format!(r#"<span class="meta-pill">★ {}</span>"#, stars));
            }
        }
        let pills_html = if pills.is_empty() {
            String::new()
        } else {
            format!(r#"<div class="meta-pills">{}</div>"#, pills)
        };

        let desc_html = repo
            .description
            .as_deref()
            .filter(|d| !d.is_empty())
            .map(|d| format!(r#"<p class="card-desc">{}</p>"#, escape_html(d)))
            .unwrap_or_default();

        let updated_label = repo
            .credit
            .as_deref()
            .filter(|c| !c.is_empty())
            .map(str::to_string)
            .or_else(|| repo.updated_at.as_deref().map(format_short_date))
            .unwrap_or_else(|| "Repository".to_string());

        html.push_str(&format!(
            r#"<a href="{}" target="_blank" rel="noopener noreferrer nofollow" class="video-card" style="padding: 24px; animation-delay: {:.3}s;"><div class="card-header-row"><h4 class="video-title" style="font-size: 16px; display: flex; align-items: center; gap: 8px;"><svg class="repo-icon" width="18" height="18" viewBox="0 0 16 16" fill="var(--text-muted)"><path fill-rule="evenodd" d="M2 2.5A2.5 2.5 0 014.5 0h8.75a.75.75 0 01.75.75v12.5a.75.75 0 01-.75.75h-2.5a.75.75 0 110-1.5h1.75v-2h-8a1 1 0 00-.714 1.7.75.75 0 01-1.072 1.05A2.495 2.495 0 012 11.5v-9zm10.5-1V9h-8c-.356 0-.694.074-1 .208V2.5a1 1 0 011-1h8zM5 12.25v3.25a.25.25 0 00.4.2l1.45-1.087a.25.25 0 01.3 0L8.6 15.7a.25.25 0 00.4-.2v-3.25a.25.25 0 00-.25-.25h-3.5a.25.25 0 00-.25.25z"></path></svg><span>{}</span></h4><button type="button" class="copy-link-btn" data-copy-url="{}" title="Copy repository URL" aria-label="Copy repository URL"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>{}{}<div class="video-stats" style="margin-top: auto; padding-top: 14px; border-top: 1px solid var(--border-subtle);"><span>{}</span><span>GitHub ↗</span></div></a>"#,
            escape_html(&href),
            delay,
            escape_html(&repo.name),
            escape_html(&href),
            desc_html,
            pills_html,
            escape_html(&updated_label),
        ));
    }

    grid.set_inner_html(&html);
}

fn render_software_grid(state: &Rc<RefCell<AppState>>, platform: &str) {
    if !is_valid_tab(platform) {
        return;
    }
    let st = state.borrow();
    let Some(grid) = document().get_element_by_id(&format!("grid-{}", platform)) else {
        return;
    };

    if let Some(err) = &st.software_sync_error {
        grid.set_inner_html(&format!(
            r#"<div class="error-banner">{}</div>"#,
            escape_html(err)
        ));
        update_count_badge(platform, 0);
        return;
    }

    let all_items = match platform {
        "windows" => &st.config.software.windows,
        "linux" => &st.config.software.linux,
        "android" => &st.config.software.android,
        _ => return,
    };

    let query_raw = st
        .search_queries
        .get(platform)
        .map(String::as_str)
        .unwrap_or("");
    let query = query_raw.to_ascii_lowercase();

    let items: Vec<&SoftwareItem> = all_items
        .iter()
        .filter(|item| {
            query.is_empty()
                || item.name.to_ascii_lowercase().contains(&query)
                || item
                    .description
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains(&query)
        })
        .collect();

    update_count_badge(platform, items.len());

    if all_items.is_empty() {
        grid.set_inner_html(
            r#"<div style="grid-column: 1/-1; padding: 40px; text-align: center; color: var(--text-secondary);">No software linked for this platform yet.</div>"#,
        );
        return;
    }

    if items.is_empty() {
        grid.set_inner_html(&format!(
            r#"<div style="grid-column: 1/-1; padding: 40px; text-align: center; color: var(--text-secondary);">No results matching "{}"</div>"#,
            escape_html(query_raw)
        ));
        return;
    }

    let mut html = String::with_capacity(items.len() * 520);
    for (idx, item) in items.iter().enumerate() {
        let primary_url = get_primary_url(item);
        let delay = ((idx as f64) * 0.02).min(0.35);

        let mut pills = String::new();
        if let Some(ver) = &item.version {
            if !ver.is_empty() {
                pills.push_str(&format!(
                    r#"<span class="meta-pill accent">{}</span>"#,
                    escape_html(ver)
                ));
            }
        }
        if let Some(size) = &item.size {
            if !size.is_empty() {
                pills.push_str(&format!(
                    r#"<span class="meta-pill">{}</span>"#,
                    escape_html(size)
                ));
            }
        }
        let pills_html = if pills.is_empty() {
            String::new()
        } else {
            format!(r#"<div class="meta-pills">{}</div>"#, pills)
        };

        let desc_html = item
            .description
            .as_deref()
            .filter(|d| !d.is_empty())
            .map(|d| format!(r#"<p class="card-desc">{}</p>"#, escape_html(d)))
            .unwrap_or_default();

        html.push_str(&format!(
            r#"<div class="video-card" style="padding: 24px; animation-delay: {:.3}s;" data-external-url="{}" tabindex="0" role="link"><div class="card-header-row"><h4 class="video-title" style="font-size: 16px;">{}</h4><button type="button" class="copy-link-btn" data-copy-url="{}" title="Copy download link" aria-label="Copy download link"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg></button></div>{}{}{}</div>"#,
            delay,
            escape_html(&primary_url),
            escape_html(&item.name),
            escape_html(&primary_url),
            desc_html,
            pills_html,
            build_download_buttons(item),
        ));
    }

    grid.set_inner_html(&html);
}

fn render_all_software_tabs(state: &Rc<RefCell<AppState>>) {
    render_github_repos(state);
    for platform in ["windows", "linux", "android"] {
        render_software_grid(state, platform);
    }
}

fn switch_tab(state: &Rc<RefCell<AppState>>, tab_id: &str, update_hash: bool) {
    if !is_valid_tab(tab_id) {
        return;
    }

    if let Ok(btns) = document().query_selector_all("button.nav-btn") {
        for i in 0..btns.length() {
            if let Some(el) = btns.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                let _ = el.class_list().remove_1("active");
            }
        }
    }
    if let Some(nav_btn) = document().get_element_by_id(&format!("nav-{}", tab_id)) {
        let _ = nav_btn.class_list().add_1("active");
    }

    if let Ok(views) = document().query_selector_all(".view-section") {
        for i in 0..views.length() {
            if let Some(el) = views.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                let _ = el.class_list().remove_1("active");
            }
        }
    }
    if let Some(tab_view) = document().get_element_by_id(&format!("tab-{}", tab_id)) {
        let _ = tab_view.class_list().add_1("active");
    }

    if let Some(main_content) = document().get_element_by_id("mainContent") {
        main_content.set_scroll_top(0);
    }

    if update_hash {
        let target_hash = format!("#{}", tab_id);
        let current_hash = window().location().hash().unwrap_or_default();
        if current_hash != target_hash {
            if let Ok(history) = window().history() {
                let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&target_hash));
            }
        }
    }

    animate_section_heading(tab_id);

    match tab_id {
        "youtube" => render_videos(state),
        "github" => render_github_repos(state),
        "windows" | "linux" | "android" => render_software_grid(state, tab_id),
        _ => {}
    }
}

fn open_player(video_id: &str, title: &str) {
    if !is_valid_video_id(video_id) {
        return;
    }
    if let Some(title_el) = document().get_element_by_id("playerTitle") {
        title_el.set_text_content(Some(if title.is_empty() { "Now Playing" } else { title }));
    }
    if let Some(iframe) = document()
        .get_element_by_id("youtubeIframe")
        .and_then(|e| e.dyn_into::<HtmlIFrameElement>().ok())
    {
        iframe.set_src(&format!(
            "https://www.youtube.com/embed/{}?autoplay=1&rel=0",
            encode_uri_component(video_id)
        ));
    }
    if let Some(modal) = document().get_element_by_id("playerModal") {
        let _ = modal.class_list().add_1("active");
    }
}

fn close_player() {
    if let Some(iframe) = document()
        .get_element_by_id("youtubeIframe")
        .and_then(|e| e.dyn_into::<HtmlIFrameElement>().ok())
    {
        iframe.set_src("");
    }
    if let Some(modal) = document().get_element_by_id("playerModal") {
        let _ = modal.class_list().remove_1("active");
    }
}

fn init_profile() {
    if let Some(name_el) = document().get_element_by_id("displayChannelName") {
        name_el.set_text_content(Some(CHANNEL_NAME));
    }
    let Some(avatar_el) = document().get_element_by_id("avatarContainer") else {
        return;
    };

    let safe_logo = sanitize_url(LOGO_URL);
    if safe_logo != "#" {
        avatar_el.set_inner_html("");
        if let Some(img) = document()
            .create_element("img")
            .ok()
            .and_then(|e| e.dyn_into::<HtmlImageElement>().ok())
        {
            img.set_src(&safe_logo);
            img.set_alt(&format!("{} Logo", CHANNEL_NAME));
            img.set_class_name("avatar");
            let avatar_clone = avatar_el.clone();
            let on_err = Closure::once(move || {
                avatar_clone.set_inner_html(DEFAULT_LOGO_SVG);
            });
            img.set_onerror(Some(on_err.as_ref().unchecked_ref()));
            on_err.forget();
            let _ = avatar_el.append_child(&img);
        }
    } else {
        avatar_el.set_inner_html(DEFAULT_LOGO_SVG);
    }
}

fn init_typewriter(state: Rc<RefCell<AppState>>) {
    let Some(el) = document().get_element_by_id("typewriterText") else {
        return;
    };

    struct TypeState {
        phrase_idx: usize,
        char_idx: usize,
        is_deleting: bool,
    }

    let initial_len = state
        .borrow()
        .site
        .typewriter_phrases
        .first()
        .map(|s| s.chars().count())
        .unwrap_or(0);
    let ts = Rc::new(RefCell::new(TypeState {
        phrase_idx: 0,
        char_idx: initial_len,
        is_deleting: true,
    }));

    let tick_fn: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let tick_clone = tick_fn.clone();
    let ts_clone = ts.clone();
    let el_clone = el.clone();
    let state_tw = state.clone();

    *tick_fn.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let phrases = state_tw.borrow().site.typewriter_phrases.clone();
        if phrases.is_empty() {
            return;
        }
        let mut s = ts_clone.borrow_mut();
        if s.phrase_idx >= phrases.len() {
            s.phrase_idx = 0;
        }
        let current = &phrases[s.phrase_idx];
        let total_chars = current.chars().count();

        let next_delay = if s.is_deleting {
            if s.char_idx > 0 {
                s.char_idx -= 1;
            }
            let sub: String = current.chars().take(s.char_idx).collect();
            el_clone.set_text_content(Some(&sub));
            if s.char_idx == 0 {
                s.is_deleting = false;
                s.phrase_idx = (s.phrase_idx + 1) % phrases.len();
                260
            } else {
                28
            }
        } else {
            s.char_idx += 1;
            let sub: String = current.chars().take(s.char_idx).collect();
            el_clone.set_text_content(Some(&sub));
            if s.char_idx >= total_chars {
                s.is_deleting = true;
                2200
            } else {
                52
            }
        };

        if let Some(cb) = tick_clone.borrow().as_ref() {
            let _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                next_delay,
            );
        }
    }) as Box<dyn FnMut()>));

    if let Some(cb) = tick_fn.borrow().as_ref() {
        let _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            2000,
        );
    };
}

async fn fetch_text(url: &str, same_origin: bool, accept: &str) -> Result<String, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(if same_origin {
        RequestMode::SameOrigin
    } else {
        RequestMode::Cors
    });
    opts.set_credentials(RequestCredentials::Omit);

    let headers = Headers::new()?;
    headers.set("Accept", accept)?;
    opts.set_headers(&headers);

    let request = Request::new_with_str_and_init(url, &opts)?;
    let resp_val = JsFuture::from(window().fetch_with_request(&request)).await?;
    let resp: Response = resp_val.dyn_into()?;
    if !resp.ok() {
        return Err(JsValue::from_str(&format!("HTTP {}", resp.status())));
    }
    let text_val = JsFuture::from(resp.text()?).await?;
    Ok(text_val.as_string().unwrap_or_default())
}

fn spawn_software_sync(state: Rc<RefCell<AppState>>) {
    render_all_software_tabs(&state);

    spawn_local(async move {
        let url = format!("software.json?t={}", Date::now() as u64);
        match fetch_text(&url, true, "application/json").await {
            Ok(body) => {
                if let Ok(parsed) = serde_json::from_str::<SoftwareData>(&body) {
                    {
                        let mut st = state.borrow_mut();
                        st.software_sync_error = None;
                        st.config.software = SoftwareData {
                            github: sanitize_software_list(parsed.github),
                            windows: sanitize_software_list(parsed.windows),
                            linux: sanitize_software_list(parsed.linux),
                            android: sanitize_software_list(parsed.android),
                        };
                        save_config(&st.config);
                    }
                    render_all_software_tabs(&state);
                }
            }
            Err(_) => {
                let has_cached = {
                    let st = state.borrow();
                    !st.config.software.github.is_empty()
                        || !st.config.software.windows.is_empty()
                        || !st.config.software.linux.is_empty()
                        || !st.config.software.android.is_empty()
                };
                if !has_cached {
                    state.borrow_mut().software_sync_error = Some(
                        "Unable to load software.json. Make sure software.json contains valid JSON and is served over HTTP/HTTPS."
                            .to_string(),
                    );
                    render_all_software_tabs(&state);
                }
            }
        }
    });
}

#[derive(Deserialize)]
struct YtSearchResp {
    items: Option<Vec<YtSearchItem>>,
}
#[derive(Deserialize)]
struct YtSearchItem {
    id: YtSearchId,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct YtSearchId {
    video_id: Option<String>,
}
#[derive(Deserialize)]
struct YtVideosResp {
    items: Option<Vec<YtVideoEntry>>,
}
#[derive(Deserialize)]
struct YtVideoEntry {
    id: String,
    snippet: Option<YtSnippet>,
    statistics: Option<YtStats>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct YtSnippet {
    title: Option<String>,
    published_at: Option<String>,
    thumbnails: Option<YtThumbnails>,
}
#[derive(Deserialize)]
struct YtThumbnails {
    high: Option<YtThumbUrl>,
    medium: Option<YtThumbUrl>,
}
#[derive(Deserialize)]
struct YtThumbUrl {
    url: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct YtStats {
    view_count: Option<String>,
}

fn render_skeletons() {
    let Some(grid) = document().get_element_by_id("videoGrid") else {
        return;
    };
    let card = r#"<div class="skeleton-card"><div class="skeleton skeleton-thumb"></div><div class="skeleton-body"><div class="skeleton skeleton-title"></div><div class="skeleton skeleton-title" style="width: 60%;"></div><div class="skeleton skeleton-meta"></div></div></div>"#;
    grid.set_inner_html(&card.repeat(6));
}

#[derive(Deserialize)]
struct YtPlaylistResp {
    items: Option<Vec<YtPlaylistItem>>,
}
#[derive(Deserialize)]
struct YtPlaylistItem {
    snippet: Option<YtPlaylistSnippet>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct YtPlaylistSnippet {
    resource_id: Option<YtResourceId>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct YtResourceId {
    video_id: Option<String>,
}

fn apply_site_config(state: &Rc<RefCell<AppState>>) {
    let st = state.borrow();
    if let Some(el) = document().get_element_by_id("displayChannelName") {
        el.set_text_content(Some(&st.site.channel_name));
    }
    if let Ok(Some(el)) = document().query_selector(".channel-handle") {
        el.set_text_content(Some(&st.site.handle));
    }
    if let Ok(Some(el)) = document().query_selector(".channel-bio") {
        el.set_text_content(Some(&st.site.bio));
    }
    if let Some(first) = st.site.typewriter_phrases.first() {
        if let Some(el) = document().get_element_by_id("typewriterText") {
            el.set_text_content(Some(first));
        }
    }
    if let Some(container) = document().get_element_by_id("topicFilters") {
        let active = st.active_topic.as_str();
        let mut html = String::with_capacity(st.site.topics.len() * 96);
        for t in &st.site.topics {
            let id_lower = t.id.to_ascii_lowercase();
            let cls = if id_lower == active {
                "topic-chip active"
            } else {
                "topic-chip"
            };
            html.push_str(&format!(
                r#"<button type="button" class="{}" data-topic="{}">{}</button>"#,
                cls,
                escape_html(&id_lower),
                escape_html(&t.label)
            ));
        }
        container.set_inner_html(&html);
    }
}

fn spawn_config_and_youtube_sync(state: Rc<RefCell<AppState>>) {
    if state.borrow().config.videos.is_empty() {
        render_skeletons();
    } else {
        render_videos(&state);
    }

    spawn_local(async move {
        // 1. Load config.json first so name, handle, bio, typewriter, topics, and YouTube settings update live
        let cfg_url = format!("config.json?t={}", Date::now() as u64);
        if let Ok(cfg_body) = fetch_text(&cfg_url, true, "application/json").await {
            if let Ok(parsed) = serde_json::from_str::<SiteConfig>(&cfg_body) {
                state.borrow_mut().site = parsed;
                apply_site_config(&state);
            }
        }

        let (api_key, channel_id) = {
            let st = state.borrow();
            (st.site.api_key.clone(), st.site.channel_id.clone())
        };

        // 2. Fetch channel uploads playlist (1 quota unit vs 100 units for search!)
        let mut ids: Vec<String> = Vec::new();
        let mut last_err: Option<String> = None;

        if let Some(rest) = channel_id.strip_prefix("UC") {
            let uploads_id = format!("UU{}", rest);
            let pl_url = format!(
                "https://www.googleapis.com/youtube/v3/playlistItems?key={}&playlistId={}&part=snippet&maxResults=30",
                encode_uri_component(&api_key),
                encode_uri_component(&uploads_id)
            );
            match fetch_text(&pl_url, false, "application/json").await {
                Ok(body) => {
                    if let Ok(pl_data) = serde_json::from_str::<YtPlaylistResp>(&body) {
                        if let Some(items) = pl_data.items {
                            ids = items
                                .into_iter()
                                .filter_map(|i| i.snippet.and_then(|s| s.resource_id).and_then(|r| r.video_id))
                                .filter(|id| is_valid_video_id(id))
                                .collect();
                        }
                    }
                }
                Err(e) => {
                    last_err = e.as_string();
                }
            }
        }

        // Fallback to search endpoint if playlistItems returned nothing
        if ids.is_empty() {
            let search_url = format!(
                "https://www.googleapis.com/youtube/v3/search?key={}&channelId={}&part=snippet,id&order=date&maxResults=30&type=video",
                encode_uri_component(&api_key),
                encode_uri_component(&channel_id)
            );
            match fetch_text(&search_url, false, "application/json").await {
                Ok(search_body) => {
                    if let Ok(search_data) = serde_json::from_str::<YtSearchResp>(&search_body) {
                        if let Some(items) = search_data.items {
                            ids = items
                                .into_iter()
                                .filter_map(|i| i.id.video_id)
                                .filter(|id| is_valid_video_id(id))
                                .collect();
                        }
                    }
                }
                Err(e) => {
                    if last_err.is_none() {
                        last_err = e.as_string();
                    }
                }
            }
        }

        if !ids.is_empty() {
            let stats_url = format!(
                "https://www.googleapis.com/youtube/v3/videos?key={}&id={}&part=snippet,statistics",
                encode_uri_component(&api_key),
                encode_uri_component(&ids.join(","))
            );
                        if let Ok(stats_body) =
                            fetch_text(&stats_url, false, "application/json").await
                        {
                            if let Ok(stats_data) =
                                serde_json::from_str::<YtVideosResp>(&stats_body)
                            {
                                if let Some(v_items) = stats_data.items {
                                    let mapped: Vec<VideoItem> = v_items
                                        .into_iter()
                                        .filter(|v| is_valid_video_id(&v.id))
                                        .map(|v| {
                                            let title = v
                                                .snippet
                                                .as_ref()
                                                .and_then(|s| s.title.as_deref())
                                                .unwrap_or("Untitled Video")
                                                .chars()
                                                .take(200)
                                                .collect();
                                            let published_at = v
                                                .snippet
                                                .as_ref()
                                                .and_then(|s| s.published_at.clone())
                                                .unwrap_or_default();
                                            let thumb = v
                                                .snippet
                                                .as_ref()
                                                .and_then(|s| s.thumbnails.as_ref())
                                                .and_then(|t| {
                                                    t.high
                                                        .as_ref()
                                                        .and_then(|u| u.url.as_deref())
                                                        .or_else(|| {
                                                            t.medium
                                                                .as_ref()
                                                                .and_then(|u| u.url.as_deref())
                                                        })
                                                })
                                                .map(sanitize_url)
                                                .unwrap_or_else(|| "#".to_string());
                                            let views = v
                                                .statistics
                                                .as_ref()
                                                .and_then(|st| st.view_count.as_deref())
                                                .and_then(|s| s.parse::<u64>().ok())
                                                .unwrap_or(0);

                                            VideoItem {
                                                id: v.id,
                                                title,
                                                views,
                                                published_at,
                                                thumbnail: thumb,
                                            }
                                        })
                                        .collect();

                                    if !mapped.is_empty() {
                                        let mut st = state.borrow_mut();
                                        st.youtube_sync_error = None;
                                        st.config.videos = mapped;
                                        save_config(&st.config);
                                    }
                                }
                            }
                        }
        } else if state.borrow().config.videos.is_empty() {
            let msg = match last_err.as_deref() {
                Some("HTTP 403") => "YouTube API returned 403 (check Google Cloud API key referrer restrictions for localhost/127.0.0.1 or quota).",
                Some("HTTP 400") => "YouTube API returned 400 (check channelId and apiKey in config.json).",
                _ => "Could not load YouTube videos (API key restricted, quota exceeded, or offline).",
            };
            state.borrow_mut().youtube_sync_error = Some(msg.to_string());
        }
        render_videos(&state);
    });
}

// High-FPS Hardware-Conscious Circuit Constellation + Data Pulses + Signal Rings
fn init_particles() {
    if let Ok(Some(mq)) = window().match_media("(prefers-reduced-motion: reduce)") {
        if mq.matches() {
            return;
        }
    }

    let Some(canvas) = document()
        .get_element_by_id("bgParticles")
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())
    else {
        return;
    };
    let Ok(Some(ctx_val)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(ctx) = ctx_val.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let w = window().inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1280.0);
    let h = window().inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(720.0);
    canvas.set_width(w as u32);
    canvas.set_height(h as u32);

    struct Particle {
        x: f64,
        y: f64,
        r: f64,
        vx: f64,
        vy: f64,
        color: &'static str,
        pulse: f64,
    }
    struct PulseBeam {
        x: f64,
        y: f64,
        len: f64,
        speed: f64,
        horizontal: bool,
        alpha: f64,
    }
    struct SignalRing {
        x: f64,
        y: f64,
        r: f64,
        max_r: f64,
        speed: f64,
        color: &'static str,
    }
    struct CanvasState {
        width: f64,
        height: f64,
        mouse_x: f64,
        mouse_y: f64,
        particles: Vec<Particle>,
        pulses: Vec<PulseBeam>,
        rings: Vec<SignalRing>,
    }

    let palette = ["#22d3ee", "#818cf8", "#34d399"];
    let max_count = if w < 768.0 { 26.0 } else { 52.0 };
    let count = ((w * h) / 24000.0).floor().clamp(20.0, max_count) as usize;

    let particles = (0..count)
        .map(|i| Particle {
            x: Math::random() * w,
            y: Math::random() * h,
            r: Math::random() * 1.9 + 0.9,
            vx: (Math::random() - 0.5) * 0.52,
            vy: (Math::random() - 0.5) * 0.52,
            color: palette[i % palette.len()],
            pulse: Math::random() * std::f64::consts::TAU,
        })
        .collect();

    let pulses = (0..6)
        .map(|_| PulseBeam {
            x: Math::random() * w,
            y: Math::random() * h,
            len: Math::random() * 85.0 + 55.0,
            speed: Math::random() * 2.5 + 1.8,
            horizontal: Math::random() > 0.5,
            alpha: Math::random() * 0.32 + 0.15,
        })
        .collect();

    let rings = (0..3)
        .map(|idx| SignalRing {
            x: Math::random() * w,
            y: Math::random() * h,
            r: (idx as f64) * 35.0,
            max_r: 115.0 + Math::random() * 60.0,
            speed: 0.48 + Math::random() * 0.22,
            color: palette[idx % palette.len()],
        })
        .collect();

    let cs = Rc::new(RefCell::new(CanvasState {
        width: w,
        height: h,
        mouse_x: -9999.0,
        mouse_y: -9999.0,
        particles,
        pulses,
        rings,
    }));

    {
        let cs_m = cs.clone();
        let on_move = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new(move |e: MouseEvent| {
            let mut s = cs_m.borrow_mut();
            s.mouse_x = e.client_x() as f64;
            s.mouse_y = e.client_y() as f64;
        }));
        let _ = window().add_event_listener_with_callback("mousemove", on_move.as_ref().unchecked_ref());
        on_move.forget();
    }
    {
        let cs_t = cs.clone();
        let on_touch = Closure::<dyn FnMut(TouchEvent)>::wrap(Box::new(move |e: TouchEvent| {
            if let Some(touch) = e.touches().get(0) {
                let mut s = cs_t.borrow_mut();
                s.mouse_x = touch.client_x() as f64;
                s.mouse_y = touch.client_y() as f64;
            }
        }));
        let _ = window().add_event_listener_with_callback("touchmove", on_touch.as_ref().unchecked_ref());
        on_touch.forget();
    }
    {
        let cs_r = cs.clone();
        let canvas_c = canvas.clone();
        let on_resize = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
            let nw = window().inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1280.0);
            let nh = window().inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(720.0);
            canvas_c.set_width(nw as u32);
            canvas_c.set_height(nh as u32);
            let mut s = cs_r.borrow_mut();
            s.width = nw;
            s.height = nh;
        }));
        let _ = window().add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref());
        on_resize.forget();
    }

    let raf_cb: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let raf_clone = raf_cb.clone();
    let cs_anim = cs.clone();

    *raf_cb.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut s = cs_anim.borrow_mut();
        let width = s.width;
        let height = s.height;
        let mouse_x = s.mouse_x;
        let mouse_y = s.mouse_y;

        ctx.clear_rect(0.0, 0.0, width, height);

        for ring in &mut s.rings {
            ring.r += ring.speed;
            if ring.r > ring.max_r {
                ring.r = 2.0;
                ring.x = Math::random() * width;
                ring.y = Math::random() * height;
            }
            let alpha = (1.0 - ring.r / ring.max_r) * 0.22;
            ctx.begin_path();
            let _ = ctx.arc(ring.x, ring.y, ring.r, 0.0, std::f64::consts::TAU);
            let stroke = match ring.color {
                "#22d3ee" => format!("rgba(34, 211, 238, {:.2})", alpha),
                "#818cf8" => format!("rgba(129, 140, 248, {:.2})", alpha),
                _ => format!("rgba(52, 211, 153, {:.2})", alpha),
            };
            ctx.set_stroke_style_str(&stroke);
            ctx.set_line_width(1.1);
            ctx.stroke();
        }

        for beam in &mut s.pulses {
            if beam.horizontal {
                beam.x += beam.speed;
                if beam.x > width + beam.len {
                    beam.x = -beam.len;
                    beam.y = Math::random() * height;
                }
                let grad = ctx.create_linear_gradient(beam.x - beam.len, beam.y, beam.x, beam.y);
                let _ = grad.add_color_stop(0.0, "rgba(34, 211, 238, 0)");
                let _ = grad.add_color_stop(1.0, &format!("rgba(34, 211, 238, {:.2})", beam.alpha));
                ctx.set_stroke_style_canvas_gradient(&grad);
                ctx.set_line_width(1.6);
                ctx.begin_path();
                ctx.move_to(beam.x - beam.len, beam.y);
                ctx.line_to(beam.x, beam.y);
                ctx.stroke();
            } else {
                beam.y += beam.speed;
                if beam.y > height + beam.len {
                    beam.y = -beam.len;
                    beam.x = Math::random() * width;
                }
                let grad = ctx.create_linear_gradient(beam.x, beam.y - beam.len, beam.x, beam.y);
                let _ = grad.add_color_stop(0.0, "rgba(129, 140, 248, 0)");
                let _ = grad.add_color_stop(1.0, &format!("rgba(129, 140, 248, {:.2})", beam.alpha));
                ctx.set_stroke_style_canvas_gradient(&grad);
                ctx.set_line_width(1.6);
                ctx.begin_path();
                ctx.move_to(beam.x, beam.y - beam.len);
                ctx.line_to(beam.x, beam.y);
                ctx.stroke();
            }
        }

        let len = s.particles.len();
        let max_dist_sq = 135.0 * 135.0;
        for i in 0..len {
            {
                let p = &mut s.particles[i];
                p.pulse += 0.03;
                p.x = (p.x + p.vx + width) % width;
                p.y = (p.y + p.vy + height) % height;

                let dx_m = p.x - mouse_x;
                let dy_m = p.y - mouse_y;
                let dist_sq_m = dx_m * dx_m + dy_m * dy_m;
                if dist_sq_m < 22500.0 && dist_sq_m > 0.0 {
                    let dist_m = dist_sq_m.sqrt();
                    let force = (150.0 - dist_m) / 150.0;
                    p.x += (dx_m / dist_m) * force * 1.15;
                    p.y += (dy_m / dist_m) * force * 1.15;
                    ctx.begin_path();
                    ctx.move_to(p.x, p.y);
                    ctx.line_to(mouse_x, mouse_y);
                    ctx.set_stroke_style_str(&format!("rgba(34, 211, 238, {:.2})", force * 0.38));
                    ctx.set_line_width(1.0);
                    ctx.stroke();
                }

                let r = (p.r + p.pulse.sin() * 0.4).max(0.5);
                ctx.begin_path();
                let _ = ctx.arc(p.x, p.y, r, 0.0, std::f64::consts::TAU);
                ctx.set_fill_style_str(p.color);
                ctx.fill();
            }

            let px = s.particles[i].x;
            let py = s.particles[i].y;
            for j in (i + 1)..len {
                let p2x = s.particles[j].x;
                let p2y = s.particles[j].y;
                let dx = px - p2x;
                let dy = py - p2y;
                let d_sq = dx * dx + dy * dy;
                if d_sq < max_dist_sq {
                    let alpha = (1.0 - d_sq.sqrt() / 135.0) * 0.26;
                    if alpha > 0.02 {
                        ctx.begin_path();
                        ctx.move_to(px, py);
                        ctx.line_to(p2x, p2y);
                        ctx.set_stroke_style_str(&format!("rgba(129, 140, 248, {:.2})", alpha));
                        ctx.set_line_width(0.85);
                        ctx.stroke();
                    }
                }
            }
        }

        if let Some(cb) = raf_clone.borrow().as_ref() {
            let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut()>));

    if let Some(cb) = raf_cb.borrow().as_ref() {
        let _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
    };
}

fn bind_event_listeners(state: &Rc<RefCell<AppState>>) {
    // Tab buttons
    if let Ok(btns) = document().query_selector_all("[data-tab]") {
        for i in 0..btns.length() {
            if let Some(el) = btns.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                let tab = el.get_attribute("data-tab").unwrap_or_default();
                let st = state.clone();
                let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
                    switch_tab(&st, &tab, true);
                }));
                let _ = el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
                cb.forget();
            }
        }
    }

    // Coming-soon social buttons
    if let Ok(btns) = document().query_selector_all("[data-coming-soon]") {
        for i in 0..btns.length() {
            if let Some(el) = btns.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                let name = el.get_attribute("data-coming-soon").unwrap_or_default();
                let st = state.clone();
                let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
                    show_toast(&st, &format!("{} link coming soon!", name));
                }));
                let _ = el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
                cb.forget();
            }
        }
    }

    // YouTube topic filter chips (delegated so config.json topics work dynamically)
    if let Some(filters) = document().get_element_by_id("topicFilters") {
        let st = state.clone();
        let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |e: Event| {
            let Some(target) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            let Ok(Some(chip)) = target.closest("[data-topic]") else {
                return;
            };
            let topic = chip.get_attribute("data-topic").unwrap_or_else(|| "all".to_string());
            st.borrow_mut().active_topic = topic.clone();
            if let Ok(all) = document().query_selector_all("[data-topic]") {
                for j in 0..all.length() {
                    if let Some(c) = all.item(j).and_then(|n| n.dyn_into::<Element>().ok()) {
                        let is_active = c.get_attribute("data-topic").as_deref() == Some(topic.as_str());
                        let _ = c.class_list().toggle_with_force("active", is_active);
                    }
                }
            }
            render_videos(&st);
        }));
        let _ = filters.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // Search inputs (+ Escape to clear)
    if let Ok(inputs) = document().query_selector_all("[data-search-tab]") {
        for i in 0..inputs.length() {
            if let Some(input) = inputs
                .item(i)
                .and_then(|n| n.dyn_into::<HtmlInputElement>().ok())
            {
                let tab = input.get_attribute("data-search-tab").unwrap_or_default();
                {
                    let st = state.clone();
                    let input_c = input.clone();
                    let tab_c = tab.clone();
                    let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
                        st.borrow_mut()
                            .search_queries
                            .insert(tab_c.clone(), input_c.value().trim().to_string());
                        match tab_c.as_str() {
                            "youtube" => render_videos(&st),
                            "github" => render_github_repos(&st),
                            _ => render_software_grid(&st, &tab_c),
                        }
                    }));
                    let _ = input.add_event_listener_with_callback("input", cb.as_ref().unchecked_ref());
                    cb.forget();
                }
                {
                    let st = state.clone();
                    let input_c = input.clone();
                    let tab_c = tab.clone();
                    let cb = Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(move |e: KeyboardEvent| {
                        if e.key() == "Escape" {
                            input_c.set_value("");
                            st.borrow_mut().search_queries.insert(tab_c.clone(), String::new());
                            match tab_c.as_str() {
                                "youtube" => render_videos(&st),
                                "github" => render_github_repos(&st),
                                _ => render_software_grid(&st, &tab_c),
                            }
                        }
                    }));
                    let _ = input.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
                    cb.forget();
                }
            }
        }
    }

    // Sort dropdown
    if let Some(trigger) = document().get_element_by_id("sortSelectTrigger") {
        let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
            if let Some(opts) = document().get_element_by_id("sortModeOptions") {
                let is_hidden = opts.class_list().toggle("hidden").unwrap_or(true);
                if let Some(tr) = document().get_element_by_id("sortSelectTrigger") {
                    let _ = tr.set_attribute("aria-expanded", &(!is_hidden).to_string());
                }
            }
        }));
        let _ = trigger.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    if let Ok(opts) = document().query_selector_all("[data-sort-val]") {
        for i in 0..opts.length() {
            if let Some(el) = opts.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                let val = el.get_attribute("data-sort-val").unwrap_or_default();
                let label = el.get_attribute("data-sort-label").unwrap_or_default();
                let st = state.clone();
                let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
                    if let Some(hidden) = document()
                        .get_element_by_id("sortMode")
                        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                    {
                        hidden.set_value(&val);
                    }
                    if let Some(disp) = document().get_element_by_id("sortModeDisplay") {
                        disp.set_text_content(Some(&label));
                    }
                    if let Some(menu) = document().get_element_by_id("sortModeOptions") {
                        let _ = menu.class_list().add_1("hidden");
                    }
                    if let Some(tr) = document().get_element_by_id("sortSelectTrigger") {
                        let _ = tr.set_attribute("aria-expanded", "false");
                    }
                    render_videos(&st);
                }));
                let _ = el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
                cb.forget();
            }
        }
    }

    // Single delegated click, keyboard, and spotlight listener on mainContent (zero per-card closure leaks!)
    if let Some(main_content) = document().get_element_by_id("mainContent") {
        {
            let st = state.clone();
            let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |e: Event| {
                let Some(target) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                    return;
                };

                if let Ok(Some(copy_btn)) = target.closest("[data-copy-url]") {
                    e.prevent_default();
                    e.stop_propagation();
                    let url = sanitize_url(&copy_btn.get_attribute("data-copy-url").unwrap_or_default());
                    if url == "#" {
                        show_toast(&st, "No download link available yet");
                    } else {
                        let clipboard = window().navigator().clipboard();
                        let st_c = st.clone();
                        spawn_local(async move {
                            let _ = JsFuture::from(clipboard.write_text(&url)).await;
                            show_toast(&st_c, "Link copied to clipboard!");
                        });
                    }
                    return;
                }

                if let Ok(Some(_)) = target.closest("a.drive-btn") {
                    return;
                }

                if let Ok(Some(video_card)) = target.closest("[data-video-id]") {
                    let vid = video_card.get_attribute("data-video-id").unwrap_or_default();
                    let title = video_card
                        .get_attribute("data-video-title")
                        .unwrap_or_default();
                    open_player(&vid, &title);
                    return;
                }

                if let Ok(Some(ext_card)) = target.closest("[data-external-url]") {
                    let url = sanitize_url(&ext_card.get_attribute("data-external-url").unwrap_or_default());
                    if url != "#" {
                        let _ = window().open_with_url_and_target_and_features(
                            &url,
                            "_blank",
                            "noopener,noreferrer",
                        );
                    }
                }
            }));
            let _ = main_content.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
            cb.forget();
        }

        // Keyboard accessibility (Enter / Space on cards)
        {
            let cb = Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(move |e: KeyboardEvent| {
                if e.key() != "Enter" && e.key() != " " {
                    return;
                }
                let Some(target) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                    return;
                };
                if let Ok(Some(video_card)) = target.closest("[data-video-id]") {
                    e.prevent_default();
                    let vid = video_card.get_attribute("data-video-id").unwrap_or_default();
                    let title = video_card.get_attribute("data-video-title").unwrap_or_default();
                    open_player(&vid, &title);
                    return;
                }
                if let Ok(Some(ext_card)) = target.closest("[data-external-url]") {
                    e.prevent_default();
                    let url = sanitize_url(&ext_card.get_attribute("data-external-url").unwrap_or_default());
                    if url != "#" {
                        let _ = window().open_with_url_and_target_and_features(&url, "_blank", "noopener,noreferrer");
                    }
                }
            }));
            let _ = main_content.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
            cb.forget();
        }

        // Single delegated spotlight mousemove listener (desktop pointer only)
        let is_touch = window()
            .match_media("(hover: none)")
            .ok()
            .flatten()
            .map(|mq| mq.matches())
            .unwrap_or(false);
        if !is_touch {
            let cb = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new(move |e: MouseEvent| {
                let Some(target) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                    return;
                };
                if let Ok(Some(card_el)) = target.closest(".video-card") {
                    if let Ok(card) = card_el.dyn_into::<web_sys::HtmlElement>() {
                        let rect = card.get_bounding_client_rect();
                        let x = (e.client_x() as f64) - rect.left();
                        let y = (e.client_y() as f64) - rect.top();
                        let style = card.style();
                        let _ = style.set_property("--mouse-x", &format!("{}px", x));
                        let _ = style.set_property("--mouse-y", &format!("{}px", y));
                    }
                }
            }));
            let _ = main_content.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref());
            cb.forget();
        }
    }

    // Modal close handlers
    if let Some(modal) = document().get_element_by_id("playerModal") {
        let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| close_player()));
        let _ = modal.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        cb.forget();
    }
    if let Some(modal_box) = document().get_element_by_id("playerModalBox") {
        let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |e: Event| {
            e.stop_propagation();
        }));
        let _ = modal_box.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        cb.forget();
    }
    if let Some(close_btn) = document().get_element_by_id("closeModalBtn") {
        let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| close_player()));
        let _ = close_btn.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // Close sort dropdown on outside click
    {
        let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |e: Event| {
            let Some(target) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            if target.closest(".custom-select-wrapper").ok().flatten().is_none() {
                if let Some(opts) = document().get_element_by_id("sortModeOptions") {
                    let _ = opts.class_list().add_1("hidden");
                }
                if let Some(tr) = document().get_element_by_id("sortSelectTrigger") {
                    let _ = tr.set_attribute("aria-expanded", "false");
                }
            }
        }));
        let _ = document().add_event_listener_with_callback("click", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // Escape closes player modal
    {
        let cb = Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(move |e: KeyboardEvent| {
            if e.key() == "Escape" {
                close_player();
            }
        }));
        let _ = document().add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    // Hash routing listener
    {
        let st = state.clone();
        let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
            let hash = window().location().hash().unwrap_or_default();
            let tab = hash.trim_start_matches('#');
            if is_valid_tab(tab) {
                switch_tab(&st, tab, false);
            }
        }));
        let _ = window().add_event_listener_with_callback("hashchange", cb.as_ref().unchecked_ref());
        cb.forget();
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    let state = Rc::new(RefCell::new(AppState::new()));

    bind_event_listeners(&state);
    init_profile();
    init_typewriter(state.clone());
    init_particles();

    let initial_hash = window().location().hash().unwrap_or_default();
    let initial_tab = initial_hash.trim_start_matches('#');
    if is_valid_tab(initial_tab) {
        switch_tab(&state, initial_tab, false);
    } else {
        animate_section_heading("youtube");
    }

    spawn_software_sync(state.clone());
    spawn_config_and_youtube_sync(state);
}
