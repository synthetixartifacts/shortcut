//! Generic text transformation command.
//!
//! Replaces `fix_grammar`, `translate_text`, and `improve_text` with a single
//! `transform_text(task, text)` command that dispatches to the configured LLM
//! provider for each task.
//!
//! Tasks: "grammar" | "translate" | "improve"
//!
//! Config read pattern: all config data is extracted before the first .await so
//! no MutexGuard is held across an async boundary.

use tauri::{AppHandle, Emitter, Manager};

/// Emit the composed prompt to the debug panel when `app_settings.debug_enabled`
/// is on. Lets the user verify that system/user prompt edits propagate to the
/// provider request. No-op when debug is off.
fn emit_prompt_debug(
    app: &AppHandle,
    task: &str,
    provider_id: &str,
    model: &str,
    messages: &[crate::providers::ChatMessage],
) {
    let debug_on = app
        .state::<crate::config::ConfigState>()
        .0
        .lock()
        .map(|c| c.app_settings.debug_enabled)
        .unwrap_or(false);
    if !debug_on {
        return;
    }

    let pretty = serde_json::to_string_pretty(messages).unwrap_or_else(|_| "<serialize error>".to_string());
    let message = format!(
        "[{}] → {}/{}\n{}",
        task, provider_id, model, pretty
    );
    let _ = app.emit("debug-log", serde_json::json!({ "level": "info", "message": message }));
}

/// Transform text using the configured LLM provider for the given task.
///
/// - task: "grammar" | "translate" | "improve"
/// - text: the selected text to transform
///
/// Reads provider assignment and prompt template from config, renders the
/// prompt with the user's text, and calls the provider's complete() method.
#[tauri::command]
pub async fn transform_text(app: AppHandle, task: String, text: String) -> Result<String, String> {
    log::info!("transform_text: task={}, text_len={}", task, text.len());

    // Extract all config data before any .await (never hold MutexGuard across await)
    let (provider_id, model, prompt, system_prompt) = {
        let state = app.state::<crate::config::ConfigState>();
        let config = state.0.lock().map_err(|e| format!("Config lock poisoned: {}", e))?;

        match task.as_str() {
            "grammar" => {
                let assignment = config.providers.task_assignments.grammar.clone();
                let prompt = config.grammar.prompt.replace("{text}", &text);
                let system_prompt = config.grammar.system_prompt.clone();
                (assignment.provider_id, assignment.model, prompt, system_prompt)
            }
            "translate" => {
                let assignment = config.providers.task_assignments.translate.clone();
                let prompt = config.translate.prompt.replace("{text}", &text);
                let system_prompt = config.translate.system_prompt.clone();
                (assignment.provider_id, assignment.model, prompt, system_prompt)
            }
            "improve" => {
                let assignment = config.providers.task_assignments.improve.clone();
                let raw = &config.improve.prompt;
                let prompt = if raw.contains("{text}") {
                    raw.replace("{text}", &text)
                } else {
                    format!("{}\n\n{}", raw, text)
                };
                let system_prompt = config.improve.system_prompt.clone();
                (assignment.provider_id, assignment.model, prompt, system_prompt)
            }
            _ => return Err(format!("Unknown task: {}. Valid tasks: grammar, translate, improve", task)),
        }
    };

    // Build messages: prepend system role only when non-empty (backward compat).
    let mut messages = Vec::with_capacity(2);
    if !system_prompt.is_empty() {
        messages.push(crate::providers::ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        });
    }
    messages.push(crate::providers::ChatMessage {
        role: "user".to_string(),
        content: prompt,
    });

    let req = crate::providers::ChatRequest {
        model,
        messages,
        image: None,
        max_tokens: None,
        temperature: Some(0.1),
    };

    let provider = crate::providers::get_llm_provider(&app, &provider_id)
        .map_err(|e| e.to_string())?;

    emit_prompt_debug(&app, &task, &provider_id, &req.model, &req.messages);

    let result = provider.complete(&req).await.map_err(|e| e.to_string())?;
    log::info!("transform_text done: task={}, result_len={}", task, result.len());
    Ok(result)
}
