<script lang="ts">
  /**
   * ApiKeyInput - Masked input for API keys with show/hide toggle
   *
   * Uses FormField + Input patterns. `saveStatus` forwards to FormField so the
   * per-field <SaveIndicator> renders below the input on auto-save.
   */
  import { FormField } from '$lib/components/ui/patterns';
  import { Input, Button } from '$lib/components/ui/primitives';
  import { t } from '$lib/i18n';
  import type { SaveStatus } from '$lib/utils/save-status.svelte';

  interface Props {
    label: string;
    value: string;
    placeholder?: string;
    error?: string | null;
    saveStatus?: SaveStatus;
    onchange: (value: string) => void;
  }

  let { label, value, placeholder = '', error, saveStatus, onchange }: Props = $props();
  let showValue = $state(false);

  const inputId = $derived(`api-key-${label.toLowerCase().replace(/\s+/g, '-')}`);
</script>

<FormField {label} id={inputId} {error} {saveStatus}>
  <Input
    type={showValue ? 'text' : 'password'}
    id={inputId}
    {value}
    {placeholder}
    monospace
    onchange={onchange}
  />
  <Button
    variant="secondary"
    onclick={() => showValue = !showValue}
    title={showValue ? t('common.hide') : t('common.show')}
  >
    {showValue ? t('common.hide') : t('common.show')}
  </Button>
</FormField>
