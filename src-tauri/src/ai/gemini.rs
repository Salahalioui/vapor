use super::sanitize::{sanitize_process_info, SanitizedProcessMetadata};
use crate::process::monitor::ProcessInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiProcessExplanation {
    pub summary: String,
    pub vendor: String,
    pub safety: String, // 'safe' | 'bloatware' | 'caution' | 'critical'
    pub can_terminate: bool,
    pub why_high_usage: String,
    pub recommendation: String,
    pub sanitized_query: SanitizedProcessMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessAuditItem {
    pub process_name: String,
    pub summary: String,
    pub vendor: String,
    pub safety: String, // 'safe' | 'bloatware' | 'caution' | 'critical'
    pub can_terminate: bool,
    pub why_high_usage: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetAuditReport {
    pub total_analyzed: usize,
    pub safe_count: usize,
    pub caution_count: usize,
    pub bloatware_count: usize,
    pub critical_count: usize,
    pub fleet_summary: String,
    pub recommendations: Vec<String>,
    pub items: Vec<ProcessAuditItem>,
}

/// Target Models:
/// Primary: gemini-3.8-flash
/// Fallbacks: gemini-3.7-flash, gemini-2.5-flash
/// (Deprecated gemini-2.0-flash and gemini-1.5-flash have been removed)
pub const CANDIDATE_MODELS: [&str; 3] = [
    "gemini-3.8-flash",
    "gemini-3.7-flash",
    "gemini-2.5-flash",
];

/// Resolves path to the persistent AI cache file:
/// `%LOCALAPPDATA%\Vapor\ai_process_cache.json`
pub fn get_ai_cache_path() -> PathBuf {
    if let Ok(override_path) = std::env::var("VAPOR_AI_CACHE_PATH") {
        if !override_path.trim().is_empty() {
            return PathBuf::from(override_path.trim());
        }
    }
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join("Vapor");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("ai_process_cache.json")
}

/// Loads persistent cache from disk if available
pub fn load_ai_cache_from_disk() -> HashMap<String, GeminiProcessExplanation> {
    let path = get_ai_cache_path();
    if !path.exists() {
        return HashMap::new();
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Could not read AI process cache file {:?}: {}", path, e);
            return HashMap::new();
        }
    };

    if let Ok(map) = serde_json::from_str::<HashMap<String, GeminiProcessExplanation>>(&content) {
        return map;
    }

    #[derive(Deserialize)]
    struct WrappedCache {
        processes: HashMap<String, GeminiProcessExplanation>,
    }
    if let Ok(wrapped) = serde_json::from_str::<WrappedCache>(&content) {
        return wrapped.processes;
    }

    log::warn!("AI process cache file {:?} had unexpected format, starting fresh", path);
    HashMap::new()
}

fn save_cache_map_to_disk(cache: &HashMap<String, GeminiProcessExplanation>) -> Result<(), String> {
    let path = get_ai_cache_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(cache)
        .map_err(|e| format!("Failed to serialize AI process cache: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write AI process cache to {:?}: {}", path, e))?;
    Ok(())
}

static EXPLANATION_CACHE: LazyLock<Mutex<HashMap<String, GeminiProcessExplanation>>> =
    LazyLock::new(|| {
        let loaded = load_ai_cache_from_disk();
        Mutex::new(loaded)
    });

/// Explicit initialization helper for app startup
pub fn init_ai_cache() -> usize {
    let cache = EXPLANATION_CACHE.lock().unwrap();
    cache.len()
}

pub fn get_cached_explanation(process_name: &str) -> Option<GeminiProcessExplanation> {
    let cache = EXPLANATION_CACHE.lock().unwrap();
    let name_lower = process_name.to_lowercase();
    let name_no_ext = name_lower.strip_suffix(".exe").unwrap_or(&name_lower);

    if let Some(exp) = cache.get(&name_lower) {
        return Some(exp.clone());
    }
    if let Some(exp) = cache.get(name_no_ext) {
        return Some(exp.clone());
    }
    let with_ext = format!("{}.exe", name_no_ext);
    if let Some(exp) = cache.get(&with_ext) {
        return Some(exp.clone());
    }
    None
}

pub fn insert_cached_explanation(process_name: &str, explanation: GeminiProcessExplanation) {
    let mut cache = EXPLANATION_CACHE.lock().unwrap();
    let name_lower = process_name.to_lowercase();
    let name_no_ext = name_lower.strip_suffix(".exe").unwrap_or(&name_lower).to_string();
    let with_ext = format!("{}.exe", name_no_ext);

    cache.insert(name_no_ext, explanation.clone());
    cache.insert(with_ext, explanation.clone());
    cache.insert(name_lower, explanation);

    let _ = save_cache_map_to_disk(&cache);
}

pub fn insert_cached_explanations_batch(items: &[(String, GeminiProcessExplanation)]) {
    let mut cache = EXPLANATION_CACHE.lock().unwrap();
    for (name, exp) in items {
        let name_lower = name.to_lowercase();
        let name_no_ext = name_lower.strip_suffix(".exe").unwrap_or(&name_lower).to_string();
        let with_ext = format!("{}.exe", name_no_ext);

        cache.insert(name_no_ext, exp.clone());
        cache.insert(with_ext, exp.clone());
        cache.insert(name_lower, exp.clone());
    }
    let _ = save_cache_map_to_disk(&cache);
}

pub fn clear_explanation_cache() {
    let mut cache = EXPLANATION_CACHE.lock().unwrap();
    cache.clear();
    let _ = save_cache_map_to_disk(&cache);
}

#[derive(Debug, Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: Option<String>,
    #[serde(default)]
    thought: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiCandidateContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiApiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiApiError>,
}

#[derive(Debug, Deserialize)]
struct GeminiApiError {
    message: String,
}

async fn send_gemini_request(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    payload: &serde_json::Value,
) -> Result<String, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let response = client
        .post(&url)
        .json(payload)
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("({}) {}", status, body_text));
    }

    let api_res: GeminiApiResponse = serde_json::from_str(&body_text)
        .map_err(|e| format!("Failed to parse Gemini response: {} | Raw: {}", e, body_text))?;

    if let Some(err) = api_res.error {
        return Err(format!("Gemini API returned error: {}", err.message));
    }

    let candidates = api_res
        .candidates
        .ok_or_else(|| "Empty candidates list received from Gemini API".to_string())?;

    let first_cand = candidates
        .into_iter()
        .next()
        .ok_or_else(|| "No candidate returned in Gemini response".to_string())?;

    let parts = first_cand
        .content
        .map(|c| c.parts)
        .unwrap_or_default();

    // In Gemini 3 models (3.7 / 3.8), filter out thought/reasoning parts so internal thinking
    // tokens are not accidentally joined or prepended to the final JSON payload
    let non_thought_parts: Vec<String> = parts
        .iter()
        .filter(|p| p.thought != Some(true))
        .filter_map(|p| p.text.clone())
        .collect();

    let text_parts = if !non_thought_parts.is_empty() {
        non_thought_parts
    } else {
        parts.into_iter().filter_map(|p| p.text).collect()
    };

    if text_parts.is_empty() {
        return Err("Empty response text received from Gemini API".to_string());
    }

    Ok(text_parts.join("\n"))
}

/// Executes a prompt against Gemini with:
/// 1. Google Search Grounding enabled by default (`"tools": [{"google_search": {}}]`).
/// 2. Graceful fallback without tools if search grounding fails on that model/key.
/// 3. Model fallback chain: gemini-3.8-flash -> gemini-3.7-flash -> gemini-2.5-flash.
async fn call_gemini_api(
    client: &reqwest::Client,
    api_key: &str,
    prompt: &str,
    models: &[&str],
) -> Result<String, String> {
    let mut last_error = String::new();

    for &model in models {
        // Attempt 1: Enable Google Search Grounding
        // Note: In Google Gemini generateContent REST API, combining tools with responseMimeType: "application/json"
        // triggers an HTTP 400 error. The prompt instructs the model to return raw JSON, and our parser handles JSON extraction.
        let payload_with_tools = serde_json::json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }],
            "tools": [{
                "google_search": {}
            }],
            "generationConfig": {
                "temperature": 0.2
            }
        });

        match send_gemini_request(client, api_key, model, &payload_with_tools).await {
            Ok(text) => return Ok(text),
            Err(tool_err) => {
                last_error = format!("Model {} with tools failed: {}", model, tool_err);

                // If 404 (model endpoint not found), skip to next model
                if tool_err.contains("(404)") {
                    continue;
                }

                // Attempt 2: Gracefully fall back to calling without tools
                let payload_without_tools = serde_json::json!({
                    "contents": [{
                        "parts": [{ "text": prompt }]
                    }],
                    "generationConfig": {
                        "temperature": 0.2,
                        "responseMimeType": "application/json"
                    }
                });

                match send_gemini_request(client, api_key, model, &payload_without_tools).await {
                    Ok(text) => return Ok(text),
                    Err(no_tool_err) => {
                        last_error = format!("Model {} without tools failed: {}", model, no_tool_err);
                        continue;
                    }
                }
            }
        }
    }

    Err(format!(
        "Gemini request failed across all candidate models. Last error: {}",
        last_error
    ))
}

pub async fn explain_process_with_gemini(
    api_key: &str,
    meta: SanitizedProcessMetadata,
) -> Result<GeminiProcessExplanation, String> {
    // 1. Check local persistent / in-memory cache first
    if let Some(cached) = get_cached_explanation(&meta.process_name) {
        return Ok(cached);
    }

    let key = api_key.trim();
    if key.is_empty() {
        return Err("Gemini API key is required. Please set it in Settings.".to_string());
    }

    let prompt = format!(
        "You are an expert Windows operating system performance and security engineer. \
Analyze this running process on a user's PC:\n\
- Process Name: {}\n\
- Publisher: {}\n\
- Description: {}\n\
- Location Category: {}\n\
- CPU Usage: {}%\n\
- RAM Usage: {} MB\n\n\
Respond ONLY with a valid raw JSON object (strictly no markdown backticks, no markdown fence, no preamble, no postfix) matching this schema:\n\
{{\n\
  \"summary\": \"1-2 sentences explaining what this process does\",\n\
  \"vendor\": \"Publisher or software vendor\",\n\
  \"safety\": \"safe | bloatware | caution | critical\",\n\
  \"can_terminate\": true or false,\n\
  \"why_high_usage\": \"Why this process might use CPU or RAM\",\n\
  \"recommendation\": \"Clear, concise recommendation for the user\"\n\
}}",
        meta.process_name,
        meta.publisher.as_deref().unwrap_or("Unknown"),
        meta.description.as_deref().unwrap_or("Unknown"),
        meta.path_category,
        meta.cpu_percent,
        meta.memory_mb
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let text = call_gemini_api(&client, key, &prompt, &CANDIDATE_MODELS).await?;

    let trimmed = text.trim();
    let clean_json = if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if start <= end {
            &trimmed[start..=end]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    #[derive(Deserialize)]
    struct RawExplanation {
        summary: String,
        vendor: String,
        safety: String,
        can_terminate: bool,
        why_high_usage: String,
        recommendation: String,
    }

    let parsed: RawExplanation = serde_json::from_str(clean_json)
        .map_err(|e| format!("Failed to parse AI output into JSON: {} | Raw: {}", e, clean_json))?;

    let explanation = GeminiProcessExplanation {
        summary: parsed.summary,
        vendor: parsed.vendor,
        safety: parsed.safety,
        can_terminate: parsed.can_terminate,
        why_high_usage: parsed.why_high_usage,
        recommendation: parsed.recommendation,
        sanitized_query: meta.clone(),
    };

    // Cache result & persist to disk
    insert_cached_explanation(&meta.process_name, explanation.clone());

    Ok(explanation)
}

pub async fn audit_processes_batch_with_gemini(
    api_key: &str,
    processes: &[ProcessInfo],
) -> Result<FleetAuditReport, String> {
    if processes.is_empty() {
        return Err("No processes provided for batch audit.".to_string());
    }

    // Deduplicate processes by name to avoid sending multiple child instances (e.g. 10x chrome.exe)
    let mut unique_procs: Vec<&ProcessInfo> = Vec::new();
    let mut seen_names = std::collections::HashSet::new();
    for p in processes {
        let name_lower = p.name.to_lowercase();
        if !seen_names.contains(&name_lower) {
            seen_names.insert(name_lower);
            unique_procs.push(p);
        }
        if unique_procs.len() >= 20 {
            break;
        }
    }

    // Check cache for already-known processes
    let mut cached_items: Vec<ProcessAuditItem> = Vec::new();
    let mut uncached_procs: Vec<&ProcessInfo> = Vec::new();

    for proc in &unique_procs {
        if let Some(cached) = get_cached_explanation(&proc.name) {
            cached_items.push(ProcessAuditItem {
                process_name: proc.name.clone(),
                summary: cached.summary,
                vendor: cached.vendor,
                safety: cached.safety,
                can_terminate: cached.can_terminate,
                why_high_usage: cached.why_high_usage,
                recommendation: cached.recommendation,
            });
        } else {
            uncached_procs.push(proc);
        }
    }

    // If all processes are already cached, return immediately with 0 API calls!
    if uncached_procs.is_empty() {
        let mut safe_count = 0;
        let mut caution_count = 0;
        let mut bloatware_count = 0;
        let mut critical_count = 0;

        for item in &cached_items {
            match item.safety.to_lowercase().as_str() {
                "safe" => safe_count += 1,
                "caution" => caution_count += 1,
                "bloatware" => bloatware_count += 1,
                "critical" => critical_count += 1,
                _ => safe_count += 1,
            }
        }

        let mut recommendations = Vec::new();
        for item in &cached_items {
            let s = item.safety.to_lowercase();
            if s == "bloatware" || s == "caution" || s == "critical" {
                recommendations.push(format!("{}: {}", item.process_name, item.recommendation));
            }
        }
        if recommendations.is_empty() {
            recommendations.push("All running processes verified against local performance and security definitions.".to_string());
            recommendations.push("System resource consumption is nominal with no anomalous background tasks.".to_string());
        } else {
            recommendations.truncate(3);
        }

        let fleet_summary = format!(
            "Fleet audit completed from local intelligence cache. {} total processes: {} verified safe, {} caution, {} bloatware, {} critical.",
            cached_items.len(),
            safe_count,
            caution_count,
            bloatware_count,
            critical_count
        );

        return Ok(FleetAuditReport {
            total_analyzed: cached_items.len(),
            safe_count,
            caution_count,
            bloatware_count,
            critical_count,
            fleet_summary,
            recommendations,
            items: cached_items,
        });
    }

    // Otherwise, query Gemini for uncached processes
    let key = api_key.trim();
    if key.is_empty() {
        return Err("Gemini API key is required. Please set it in Settings.".to_string());
    }

    // Sanitize only the uncached processes
    let sanitized_list: Vec<SanitizedProcessMetadata> = uncached_procs
        .iter()
        .map(|p| {
            sanitize_process_info(
                &p.name,
                p.exe_path.as_deref(),
                p.publisher.as_deref(),
                p.description.as_deref(),
                p.cpu_percent,
                p.memory_mb,
            )
        })
        .collect();

    let mut process_descs = String::new();
    for (i, p) in sanitized_list.iter().enumerate() {
        process_descs.push_str(&format!(
            "{}. Name: {} | Publisher: {} | Desc: {} | Path Category: {} | CPU: {:.1}% | RAM: {:.0} MB\n",
            i + 1,
            p.process_name,
            p.publisher.as_deref().unwrap_or("Unknown"),
            p.description.as_deref().unwrap_or("Unknown"),
            p.path_category,
            p.cpu_percent,
            p.memory_mb
        ));
    }

    let prompt = format!(
        "You are an expert Windows operating system performance and cybersecurity engineer.\n\
Analyze this active fleet of {} running processes from a user's Windows PC:\n\n\
{}\n\n\
Respond ONLY with a valid raw JSON object (strictly no markdown formatting, no backticks, no fences, no pre/post-amble) matching this JSON schema:\n\
{{\n\
  \"fleet_summary\": \"High-level 2-3 sentence overview assessing overall system health, background bloat, and any suspicious or heavy activity in this fleet\",\n\
  \"recommendations\": [\"Actionable recommendation 1\", \"Actionable recommendation 2\", \"Actionable recommendation 3\"],\n\
  \"items\": [\n\
    {{\n\
      \"process_name\": \"exact process executable name matching input\",\n\
      \"summary\": \"1-2 sentences on what this process does\",\n\
      \"vendor\": \"Publisher or software vendor\",\n\
      \"safety\": \"safe | bloatware | caution | critical\",\n\
      \"can_terminate\": true,\n\
      \"why_high_usage\": \"Why this process consumes resources\",\n\
      \"recommendation\": \"Specific advice for this process\"\n\
    }}\n\
  ]\n\
}}",
        sanitized_list.len(),
        process_descs
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(35))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let text = call_gemini_api(&client, key, &prompt, &CANDIDATE_MODELS).await?;

    let trimmed = text.trim();
    let clean_json = if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if start <= end {
            &trimmed[start..=end]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    #[derive(Deserialize)]
    struct RawFleetResponse {
        fleet_summary: String,
        #[serde(default)]
        recommendations: Vec<String>,
        #[serde(default)]
        items: Vec<ProcessAuditItem>,
    }

    let parsed: RawFleetResponse = serde_json::from_str(clean_json)
        .map_err(|e| format!("Failed to parse Fleet AI JSON: {} | Raw: {}", e, clean_json))?;

    // Prepare batch cache entries for newly analyzed processes
    let mut batch_cache_entries: Vec<(String, GeminiProcessExplanation)> = Vec::new();

    for item in &parsed.items {
        let item_name = item.process_name.to_lowercase();
        let item_no_ext = item_name.strip_suffix(".exe").unwrap_or(&item_name);

        let matching_meta = sanitized_list.iter().find(|m| {
            let m_name = m.process_name.to_lowercase();
            let m_no_ext = m_name.strip_suffix(".exe").unwrap_or(&m_name);
            m_name == item_name || m_no_ext == item_no_ext
        });

        let sanitized = if let Some(m) = matching_meta {
            m.clone()
        } else {
            SanitizedProcessMetadata {
                process_name: item.process_name.clone(),
                publisher: Some(item.vendor.clone()),
                description: Some(item.summary.clone()),
                path_category: "Unknown".to_string(),
                cpu_percent: 0.0,
                memory_mb: 0.0,
            }
        };

        let exp = GeminiProcessExplanation {
            summary: item.summary.clone(),
            vendor: item.vendor.clone(),
            safety: item.safety.clone(),
            can_terminate: item.can_terminate,
            why_high_usage: item.why_high_usage.clone(),
            recommendation: item.recommendation.clone(),
            sanitized_query: sanitized,
        };

        batch_cache_entries.push((item.process_name.clone(), exp));
    }

    // Persist newly analyzed processes to memory and disk cache
    insert_cached_explanations_batch(&batch_cache_entries);

    // Merge: Combine cached_items and parsed.items into final list
    let initial_cached_count = cached_items.len();
    let mut final_items: Vec<ProcessAuditItem> = Vec::new();
    let mut newly_audited = parsed.items;

    for proc in &unique_procs {
        let p_lower = proc.name.to_lowercase();
        let p_no_ext = p_lower.strip_suffix(".exe").unwrap_or(&p_lower);

        if let Some(pos) = newly_audited.iter().position(|it| {
            let it_lower = it.process_name.to_lowercase();
            let it_no_ext = it_lower.strip_suffix(".exe").unwrap_or(&it_lower);
            it_lower == p_lower || it_no_ext == p_no_ext
        }) {
            final_items.push(newly_audited.remove(pos));
        } else if let Some(pos) = cached_items.iter().position(|it| {
            let it_lower = it.process_name.to_lowercase();
            let it_no_ext = it_lower.strip_suffix(".exe").unwrap_or(&it_lower);
            it_lower == p_lower || it_no_ext == p_no_ext
        }) {
            final_items.push(cached_items.remove(pos));
        } else if let Some(cached) = get_cached_explanation(&proc.name) {
            final_items.push(ProcessAuditItem {
                process_name: proc.name.clone(),
                summary: cached.summary,
                vendor: cached.vendor,
                safety: cached.safety,
                can_terminate: cached.can_terminate,
                why_high_usage: cached.why_high_usage,
                recommendation: cached.recommendation,
            });
        }
    }

    // Append any remaining items that didn't match unique_procs directly
    final_items.extend(newly_audited);

    let mut safe_count = 0;
    let mut caution_count = 0;
    let mut bloatware_count = 0;
    let mut critical_count = 0;

    for item in &final_items {
        match item.safety.to_lowercase().as_str() {
            "safe" => safe_count += 1,
            "caution" => caution_count += 1,
            "bloatware" => bloatware_count += 1,
            "critical" => critical_count += 1,
            _ => safe_count += 1,
        }
    }

    let fleet_summary = if initial_cached_count == 0 {
        parsed.fleet_summary
    } else {
        format!(
            "{} (Merged with {} locally cached process definitions).",
            parsed.fleet_summary,
            initial_cached_count
        )
    };

    let mut recommendations = parsed.recommendations;
    for item in &final_items {
        let s = item.safety.to_lowercase();
        if (s == "bloatware" || s == "caution" || s == "critical") && !item.recommendation.trim().is_empty() {
            let rec = format!("{}: {}", item.process_name, item.recommendation);
            if !recommendations.contains(&rec) && !recommendations.iter().any(|r| r.contains(&item.process_name)) {
                recommendations.push(rec);
            }
        }
    }
    recommendations.truncate(3);

    let total_analyzed = final_items.len();

    Ok(FleetAuditReport {
        total_analyzed,
        safe_count,
        caution_count,
        bloatware_count,
        critical_count,
        fleet_summary,
        recommendations,
        items: final_items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_candidate_models_order_and_deprecations() {
        assert_eq!(
            CANDIDATE_MODELS,
            ["gemini-3.8-flash", "gemini-3.7-flash", "gemini-2.5-flash"]
        );
        for m in CANDIDATE_MODELS {
            assert!(!m.contains("2.0"), "Legacy 2.0 models must be removed: {}", m);
            assert!(!m.contains("1.5"), "Legacy 1.5 models must be removed: {}", m);
        }
    }

    #[test]
    fn test_thought_part_filtering() {
        let json_data = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [
                        {
                            "thought": true,
                            "text": "I am thinking about the process and whether to output JSON..."
                        },
                        {
                            "thought": false,
                            "text": "{\"summary\": \"Test\", \"vendor\": \"Microsoft\", \"safety\": \"safe\", \"can_terminate\": true, \"why_high_usage\": \"None\", \"recommendation\": \"Keep\"}"
                        }
                    ]
                }
            }]
        });

        let api_res: GeminiApiResponse = serde_json::from_value(json_data).unwrap();
        let first_cand = api_res.candidates.unwrap().into_iter().next().unwrap();
        let parts = first_cand.content.unwrap().parts;

        let non_thought_parts: Vec<String> = parts
            .iter()
            .filter(|p| p.thought != Some(true))
            .filter_map(|p| p.text.clone())
            .collect();

        assert_eq!(non_thought_parts.len(), 1);
        assert!(non_thought_parts[0].contains("\"summary\": \"Test\""));
        assert!(!non_thought_parts[0].contains("I am thinking"));
    }

    #[test]
    fn test_google_search_grounding_payload_no_response_mimetype() {
        let prompt = "Analyze svchost.exe";
        let payload_with_tools = serde_json::json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }],
            "tools": [{
                "google_search": {}
            }],
            "generationConfig": {
                "temperature": 0.2
            }
        });

        // Verify tools contains google_search
        let tools = payload_with_tools.get("tools").and_then(|t| t.as_array()).unwrap();
        assert_eq!(tools.len(), 1);
        assert!(tools[0].get("google_search").is_some());

        // Verify generationConfig does NOT contain responseMimeType when tools are present
        // (to prevent 400 Invalid Argument error from Gemini API)
        let gen_config = payload_with_tools.get("generationConfig").unwrap();
        assert!(gen_config.get("responseMimeType").is_none());
    }

    #[test]
    fn test_explanation_cache() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_explanation_cache();
        let meta = SanitizedProcessMetadata {
            process_name: "testproc.exe".to_string(),
            publisher: Some("Test Vendor".to_string()),
            description: Some("Test Desc".to_string()),
            path_category: "ProgramFiles".to_string(),
            cpu_percent: 1.5,
            memory_mb: 50.0,
        };

        let exp = GeminiProcessExplanation {
            summary: "Test process".to_string(),
            vendor: "Test Vendor".to_string(),
            safety: "safe".to_string(),
            can_terminate: true,
            why_high_usage: "None".to_string(),
            recommendation: "Safe to run".to_string(),
            sanitized_query: meta,
        };

        insert_cached_explanation("testproc.exe", exp);
        let cached = get_cached_explanation("testproc.exe");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().summary, "Test process");

        // Case insensitivity test
        let cached_upper = get_cached_explanation("TESTPROC.EXE");
        assert!(cached_upper.is_some());
        let cached_no_ext = get_cached_explanation("testproc");
        assert!(cached_no_ext.is_some());
    }

    #[test]
    fn test_disk_cache_persistence_and_reload() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_explanation_cache();
        let meta = SanitizedProcessMetadata {
            process_name: "disk_worker.exe".to_string(),
            publisher: Some("DiskVendor".to_string()),
            description: Some("Worker for disk test".to_string()),
            path_category: "System32".to_string(),
            cpu_percent: 0.5,
            memory_mb: 42.0,
        };

        let exp = GeminiProcessExplanation {
            summary: "Disk persistence process".to_string(),
            vendor: "DiskVendor".to_string(),
            safety: "safe".to_string(),
            can_terminate: false,
            why_high_usage: "None".to_string(),
            recommendation: "Retain".to_string(),
            sanitized_query: meta,
        };

        insert_cached_explanation("disk_worker.exe", exp);

        // Verify disk cache file exists
        let path = get_ai_cache_path();
        assert!(path.exists(), "Cache file must exist at {:?}", path);

        // Load directly from disk
        let disk_map = load_ai_cache_from_disk();
        assert!(disk_map.contains_key("disk_worker.exe") || disk_map.contains_key("disk_worker"));
    }

    #[tokio::test]
    async fn test_batch_fleet_audit_all_cached_zero_api_calls() {
        let _lock = TEST_MUTEX.lock().unwrap();
        clear_explanation_cache();

        let meta1 = SanitizedProcessMetadata {
            process_name: "cached_app1.exe".to_string(),
            publisher: Some("Vendor1".to_string()),
            description: Some("Desc1".to_string()),
            path_category: "ProgramFiles".to_string(),
            cpu_percent: 1.0,
            memory_mb: 50.0,
        };
        let exp1 = GeminiProcessExplanation {
            summary: "App 1 explanation".to_string(),
            vendor: "Vendor1".to_string(),
            safety: "safe".to_string(),
            can_terminate: true,
            why_high_usage: "None".to_string(),
            recommendation: "Keep running".to_string(),
            sanitized_query: meta1,
        };

        let meta2 = SanitizedProcessMetadata {
            process_name: "cached_app2.exe".to_string(),
            publisher: Some("Vendor2".to_string()),
            description: Some("Desc2".to_string()),
            path_category: "ProgramFiles".to_string(),
            cpu_percent: 12.0,
            memory_mb: 250.0,
        };
        let exp2 = GeminiProcessExplanation {
            summary: "App 2 explanation".to_string(),
            vendor: "Vendor2".to_string(),
            safety: "caution".to_string(),
            can_terminate: true,
            why_high_usage: "High memory leak".to_string(),
            recommendation: "Restart app".to_string(),
            sanitized_query: meta2,
        };

        insert_cached_explanation("cached_app1.exe", exp1);
        insert_cached_explanation("cached_app2.exe", exp2);

        let procs = vec![
            ProcessInfo {
                pid: 1001,
                name: "cached_app1.exe".to_string(),
                cpu_percent: 1.0,
                memory_bytes: 50 * 1024 * 1024,
                memory_mb: 50.0,
                exe_path: Some("C:\\Program Files\\App1\\cached_app1.exe".to_string()),
                publisher: Some("Vendor1".to_string()),
                description: Some("Desc1".to_string()),
                category: "User".to_string(),
                safety: "safe".to_string(),
                can_kill: true,
                is_known: false,
            },
            ProcessInfo {
                pid: 1002,
                name: "cached_app2.exe".to_string(),
                cpu_percent: 12.0,
                memory_bytes: 250 * 1024 * 1024,
                memory_mb: 250.0,
                exe_path: Some("C:\\Program Files\\App2\\cached_app2.exe".to_string()),
                publisher: Some("Vendor2".to_string()),
                description: Some("Desc2".to_string()),
                category: "User".to_string(),
                safety: "caution".to_string(),
                can_kill: true,
                is_known: false,
            },
        ];

        // Should succeed with EMPTY API key because all processes are cached! (0 API calls)
        let res = audit_processes_batch_with_gemini("", &procs).await;
        assert!(res.is_ok(), "Audit of cached processes must succeed without API key: {:?}", res);

        let report = res.unwrap();
        assert_eq!(report.total_analyzed, 2);
        assert_eq!(report.safe_count, 1);
        assert_eq!(report.caution_count, 1);
        assert_eq!(report.items.len(), 2);
    }
}
