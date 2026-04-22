import { invokeWithErrorHandling } from './core';

interface SelectionResult {
  text: string;
  format: 'markdown' | 'plain';
}

export async function pasteText(text: string): Promise<void> {
  await invokeWithErrorHandling<void>('paste_text', { text });
}

export async function getSelectionWithFormat(): Promise<SelectionResult> {
  return invokeWithErrorHandling<SelectionResult>('get_selection_with_format');
}

export async function pasteFormatted(text: string, format: string): Promise<void> {
  await invokeWithErrorHandling<void>('paste_formatted', { text, format });
}
