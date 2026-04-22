<script lang="ts">
  /**
   * TextInput - Simple labeled text input for settings forms
   *
   * Uses FormField + Input patterns. `saveStatus` forwards to FormField so the
   * per-field <SaveIndicator> renders below the input on auto-save.
   */
  import { FormField } from '$lib/components/ui/patterns';
  import { Input } from '$lib/components/ui/primitives';
  import type { SaveStatus } from '$lib/utils/save-status.svelte';

  interface Props {
    label: string;
    value: string;
    placeholder?: string;
    error?: string | null;
    hint?: string;
    saveStatus?: SaveStatus;
    onchange: (value: string) => void;
  }

  let { label, value, placeholder = '', error, hint, saveStatus, onchange }: Props = $props();

  const inputId = $derived(`text-${label.toLowerCase().replace(/\s+/g, '-')}`);
</script>

<FormField {label} id={inputId} {error} {hint} {saveStatus}>
  <Input
    type="text"
    id={inputId}
    {value}
    {placeholder}
    onchange={onchange}
  />
</FormField>
