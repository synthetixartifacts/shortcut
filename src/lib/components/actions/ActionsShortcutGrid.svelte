<script lang="ts">
  import { SettingsLinkCard } from '$lib/components/ui/patterns';
  import { getShortcutDisplay } from '$lib/features/shortcuts';
  import { appState } from '$lib/state/app.svelte';
  import { t } from '$lib/i18n';

  interface Props {
    platform: 'macos' | 'windows' | 'linux';
  }

  let { platform }: Props = $props();

  const items = $derived([
    { key: 'dictation', href: '/actions/dictation', icon: '🎤',
      title: t('actions.dictation_title'), desc: t('actions.dictation_desc') },
    { key: 'grammar', href: '/actions/grammar', icon: '📝',
      title: t('actions.grammar_title'), desc: t('actions.grammar_desc') },
    { key: 'translate', href: '/actions/translate', icon: '🌐',
      title: t('actions.translate_title'), desc: t('actions.translate_desc') },
    { key: 'improve', href: '/actions/improve', icon: '✨',
      title: t('actions.improve_title'), desc: t('actions.improve_desc') },
    { key: 'screen_question', href: '/actions/screen-question', icon: '📷',
      title: t('actions.screen_question_title'), desc: t('actions.screen_question_desc') },
  ]);
</script>

<div class="grid">
  {#each items as item (item.key)}
    <SettingsLinkCard
      href={item.href}
      icon={item.icon}
      title={item.title}
      description={item.desc}
      hint={getShortcutDisplay(item.key, platform)}
      active={item.key === 'dictation' && appState.isRecording}
    />
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--spacing-sm);
  }

  @media (max-width: 720px) {
    .grid { grid-template-columns: 1fr; }
  }
</style>
