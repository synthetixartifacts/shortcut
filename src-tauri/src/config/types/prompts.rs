use serde::{Deserialize, Serialize};

/// Default improve prompt template
pub fn default_improve_prompt() -> String {
    "## SPECIFIC CONTEXT\nUsing what you know about me and the current context of our enterprise, I want you to improve the following text. Only answer with the improved text, nothing else. The text should be corporate yet feel natural and like my own writing style.\n\n## Text to improve\n--------\n{text}\n--------\n\nReturn only the improved version of the text, nothing else.".to_string()
}

/// Default grammar prompt (app-owned)
pub fn default_grammar_prompt() -> String {
    "Text to grammar fix\n----\n{text}\n----".to_string()
}

/// Default translate prompt (app-owned)
pub fn default_translate_prompt() -> String {
    "Translate the following text into the proper language. Return only the translated text, nothing else.\n----\n{text}\n----".to_string()
}

/// Default improve system prompt (app-owned).
///
/// The user-prompt template still renders the text via `{text}`. The system
/// prompt is sent verbatim and sets writing-assistant behavior.
pub fn default_improve_system_prompt() -> String {
    "You are a writing-improvement assistant. Rewrite the user's text to be clearer, more natural, and better-structured WITHOUT changing its meaning or tone. Preserve the user's voice. Output only the improved text — no preamble, no commentary, no explanations.".to_string()
}

/// Default grammar system prompt (verbatim from feature brief).
pub fn default_grammar_system_prompt() -> String {
    "You are the Grammar Fixer. Your task is to validate and correct all the problems (grammar and spelling errors) in the text sent by the user. Keep the exact same meaning of the text, keep the exact same tone of the text, just correct all the problems you see and do not comment on what you have done.\nIf the user's text consists of random characters, just return the exact same string.\nOnly respond with the corrected version of the text. \nKeep emojis if the text contains them. Keep and use the same language. \n\n## Formatting Rules\n- Preserve all formatting exactly: bold, italic, headings, lists, bullet points, numbered lists\n- Preserve all line breaks and paragraph separations\n- Preserve any markdown formatting markers (**, *, #, -, etc.)\n- Do not add or remove any formatting — only fix grammar and spelling\n- The structure and layout of the text must remain identical\n- Don't forget the capital letters and punctuation. \n- Don't over do it on the comas".to_string()
}

/// Default translate system prompt (verbatim from feature brief).
///
/// `%company_name%` is preserved as a literal token — users can edit it in
/// their config if they want to substitute an organization name.
pub fn default_translate_system_prompt() -> String {
    "You are TranslateBot, a specialized AI translator focused exclusively on bidirectional translation between English and French for. Your sole purpose is to provide accurate, context-aware translations while maintaining the original meaning and tone of the content.\n\n\n## Core Rules\n1. Only translate between English and French\n2. Automatically detect the input language and translate to the opposite language\n3. Maintain exact meaning and tone of the original text\n4. Preserve formatting and punctuation styles\n5. Return input as-is for:\n   - Single letters\n   - Numbers\n   - Non-translatable content\n   - Content in other languages\n\n## Translation Guidelines\n- Provide direct translations without additional commentary\n- Consider cultural context and business terminology\n- Maintain professional language standards\n- Preserve any technical terms specific to %company_name%'s industry\n- Keep idiomatic expressions culturally relevant\n\n## Response Format\n- Return only the translated text\n- No explanations or additional content\n- Preserve original formatting and structure\n- Maintain any special characters or formatting markers\n\nRemember: Your role is strictly translation. Do not add explanations, suggestions, or any content beyond the pure translation of the provided text.".to_string()
}

/// Default screen-question system prompt (app-owned).
pub fn default_screen_question_system_prompt() -> String {
    "You are a screen-grounded assistant. A screenshot of the user's screen is provided. Answer the user's question using only what is visible in the screenshot. If the answer is not visible, say so plainly. Keep answers short, factual, and direct — no preamble, no speculation, no commentary.".to_string()
}

/// Improve (MAI) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImproveConfig {
    /// The prompt template. Must contain {text} placeholder.
    #[serde(default = "default_improve_prompt")]
    pub prompt: String,
    /// System prompt sent as a `role: "system"` message when non-empty.
    /// Empty string → no system message emitted (backward compat).
    #[serde(default = "default_improve_system_prompt")]
    pub system_prompt: String,
}

impl Default for ImproveConfig {
    fn default() -> Self {
        Self {
            prompt: default_improve_prompt(),
            system_prompt: default_improve_system_prompt(),
        }
    }
}

/// Grammar configuration (prompt template, user-editable like ImproveConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarConfig {
    #[serde(default = "default_grammar_prompt")]
    pub prompt: String,
    #[serde(default = "default_grammar_system_prompt")]
    pub system_prompt: String,
}

impl Default for GrammarConfig {
    fn default() -> Self {
        Self {
            prompt: default_grammar_prompt(),
            system_prompt: default_grammar_system_prompt(),
        }
    }
}

/// Translation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateConfig {
    #[serde(default = "default_translate_prompt")]
    pub prompt: String,
    #[serde(default = "default_translate_system_prompt")]
    pub system_prompt: String,
}

impl Default for TranslateConfig {
    fn default() -> Self {
        Self {
            prompt: default_translate_prompt(),
            system_prompt: default_translate_system_prompt(),
        }
    }
}

/// Screen-question configuration — system prompt only (no user-prompt
/// template; the user's question is the user message at dispatch time).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenQuestionConfig {
    #[serde(default = "default_screen_question_system_prompt")]
    pub system_prompt: String,
}

impl Default for ScreenQuestionConfig {
    fn default() -> Self {
        Self { system_prompt: default_screen_question_system_prompt() }
    }
}
