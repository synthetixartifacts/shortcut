<script lang="ts">
  /**
   * Onboarding — LLM Provider Step
   *
   * Collects API keys for OpenAI, Anthropic, Gemini, Grok, and the Local LLM
   * chat-completion URL (Ollama / LM Studio / LocalAI / vLLM / llama.cpp — any
   * compatible endpoint). All fields are optional — the user can skip the
   * entire step. No discovery probing happens here (D9): we only persist
   * whatever the user typed and move on.
   */
  import { t } from '$lib/i18n';

  interface Props {
    openaiKey: string;
    anthropicKey: string;
    geminiKey: string;
    grokKey: string;
    localUrl: string;
    isSaving: boolean;
    onContinue: () => void;
    onSkip: () => void;
  }

  let {
    openaiKey = $bindable(),
    anthropicKey = $bindable(),
    geminiKey = $bindable(),
    grokKey = $bindable(),
    localUrl = $bindable(),
    isSaving,
    onContinue,
    onSkip,
  }: Props = $props();
</script>

<div class="provider-grid">
  <label class="provider-row">
    <span class="provider-name">OpenAI</span>
    <p class="provider-desc">{t('providers.openai_desc')}</p>
    <input
      class="key-input"
      type="password"
      placeholder={t('settings.field_openai_key_placeholder')}
      bind:value={openaiKey}
    />
  </label>
  <label class="provider-row">
    <span class="provider-name">Anthropic</span>
    <p class="provider-desc">{t('providers.anthropic_desc')}</p>
    <input
      class="key-input"
      type="password"
      placeholder={t('settings.field_anthropic_key_placeholder')}
      bind:value={anthropicKey}
    />
  </label>
  <label class="provider-row">
    <span class="provider-name">Google Gemini</span>
    <p class="provider-desc">{t('providers.gemini_desc')}</p>
    <input
      class="key-input"
      type="password"
      placeholder={t('settings.field_gemini_key_placeholder')}
      bind:value={geminiKey}
    />
  </label>
  <label class="provider-row">
    <span class="provider-name">Grok (xAI)</span>
    <p class="provider-desc">{t('providers.grok_desc')}</p>
    <input
      class="key-input"
      type="password"
      placeholder={t('settings.field_grok_key_placeholder')}
      bind:value={grokKey}
    />
  </label>
  <label class="provider-row">
    <span class="provider-name">
      {t('settings.provider_local')} LLM
      <span class="badge-local">Local</span>
    </span>
    <p class="provider-desc">{t('providers.local_desc')}</p>
    <span class="provider-field-label">{t('settings.field_local_url')}</span>
    <input
      class="key-input"
      type="text"
      placeholder={t('settings.field_local_url_placeholder')}
      bind:value={localUrl}
    />
  </label>
</div>

<div class="actions">
  <button class="btn-primary" onclick={onContinue} disabled={isSaving}>
    {isSaving ? t('common.saving') : t('onboarding.continue')}
  </button>
  <button class="btn-skip" onclick={onSkip}>{t('onboarding.skip')}</button>
</div>

<style>
  .provider-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);
  }

  .provider-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    background: var(--color-surface);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-md);
    padding: var(--spacing-md);
    cursor: default;
  }

  .provider-name {
    font-weight: 600;
    font-size: 0.95rem;
    color: var(--color-text);
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .badge-local {
    font-size: 0.7rem;
    font-weight: 500;
    padding: 1px 6px;
    border-radius: 9999px;
    background: var(--color-success);
    color: var(--color-text-on-status);
  }

  .provider-desc {
    margin: 0 0 4px;
    font-size: 0.8rem;
    color: var(--color-text-muted);
    line-height: 1.45;
  }

  .provider-field-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .key-input {
    width: 100%;
    margin-top: auto;
    padding: 6px var(--spacing-sm);
    font-size: 0.85rem;
    font-family: monospace;
    background: var(--color-background);
    border: 1px solid var(--color-kbd-border);
    border-radius: var(--border-radius-sm);
    color: var(--color-text);
    box-sizing: border-box;
  }

  .key-input:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .btn-primary {
    width: 100%;
    max-width: 320px;
    padding: 10px var(--spacing-lg);
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--color-text-on-primary);
    background: var(--color-primary);
    border: none;
    border-radius: var(--border-radius-md);
    cursor: pointer;
    transition: opacity 0.15s ease;
  }

  .btn-primary:hover:not(:disabled) { opacity: 0.9; }
  .btn-primary:disabled { opacity: 0.5; cursor: default; }

  .btn-skip {
    background: none;
    border: none;
    color: var(--color-text-muted);
    font-size: 0.85rem;
    cursor: pointer;
    padding: var(--spacing-sm);
    text-decoration: underline;
    transition: color 0.15s ease;
  }

  .btn-skip:hover { color: var(--color-text); }

  @media (max-width: 720px) {
    .provider-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
