/**
 * Microphone Permission Service
 *
 * Handles checking and requesting microphone permissions with
 * cross-browser and cross-platform support.
 */

import { log, logError } from '$lib/utils/logger';

/** Permission states */
export type PermissionState = 'granted' | 'denied' | 'prompt' | 'unknown';

/** Platform types */
type Platform = 'windows' | 'macos' | 'linux' | 'unknown';

/** Full permission status */
interface MicrophonePermissionStatus {
  state: PermissionState;
  isSupported: boolean;
  platform: Platform;
}

/**
 * Detect current platform
 */
function detectPlatform(): Platform {
  const userAgent = navigator.userAgent.toLowerCase();

  if (userAgent.includes('win')) return 'windows';
  if (userAgent.includes('mac')) return 'macos';
  if (userAgent.includes('linux')) return 'linux';

  return 'unknown';
}

/**
 * Check if MediaDevices API is supported
 */
function isMediaDevicesSupported(): boolean {
  return !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia);
}

/**
 * Check microphone permission state using Permissions API
 * Falls back to 'unknown' if Permissions API not supported
 */
export async function checkMicrophonePermission(): Promise<MicrophonePermissionStatus> {
  const platform = detectPlatform();
  const isSupported = isMediaDevicesSupported();

  if (!isSupported) {
    await log('[MicrophonePermission] MediaDevices API not supported');
    return {
      state: 'unknown',
      isSupported: false,
      platform,
    };
  }

  // Try Permissions API first (not supported in all browsers)
  if (navigator.permissions) {
    try {
      const result = await navigator.permissions.query({
        name: 'microphone' as PermissionName,
      });

      await log(`[MicrophonePermission] Permission state: ${result.state}`);

      return {
        state: result.state as PermissionState,
        isSupported: true,
        platform,
      };
    } catch (e) {
      // Permissions API doesn't support 'microphone' in this browser
      // This is common in Firefox and Safari
      await log('[MicrophonePermission] Permissions API not available for microphone, will probe');
    }
  }

  // Fallback: Try to enumerate devices to detect permission state
  // If we can get device labels, permission was granted
  // If labels are empty strings, permission was never asked or denied
  try {
    const devices = await navigator.mediaDevices.enumerateDevices();
    const audioDevices = devices.filter(d => d.kind === 'audioinput');

    if (audioDevices.length === 0) {
      // No audio devices at all
      return {
        state: 'unknown',
        isSupported: true,
        platform,
      };
    }

    // Check if we have device labels (indicates permission was granted)
    const hasLabels = audioDevices.some(d => d.label && d.label.length > 0);

    if (hasLabels) {
      await log('[MicrophonePermission] Device labels available, permission granted');
      return {
        state: 'granted',
        isSupported: true,
        platform,
      };
    }

    // No labels - permission either not asked or denied
    // Return 'prompt' to indicate user needs to grant permission
    await log('[MicrophonePermission] No device labels, permission needed');
    return {
      state: 'prompt',
      isSupported: true,
      platform,
    };
  } catch (e) {
    await logError('[MicrophonePermission] Failed to enumerate devices', e);
    return {
      state: 'unknown',
      isSupported: true,
      platform,
    };
  }
}

/**
 * Request microphone permission
 * Returns the resulting permission state
 */
export async function requestMicrophonePermission(): Promise<PermissionState> {
  await log('[MicrophonePermission] Requesting permission...');

  if (!isMediaDevicesSupported()) {
    await log('[MicrophonePermission] MediaDevices not supported');
    return 'denied';
  }

  try {
    // Request access - this will trigger the permission prompt
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });

    // Immediately stop the stream - we just needed permission
    stream.getTracks().forEach(track => track.stop());

    await log('[MicrophonePermission] Permission granted');
    return 'granted';
  } catch (e) {
    const error = e as DOMException;

    if (error.name === 'NotAllowedError') {
      await log('[MicrophonePermission] Permission denied by user');
      return 'denied';
    }

    if (error.name === 'NotFoundError') {
      await log('[MicrophonePermission] No microphone found');
      return 'denied';
    }

    await logError('[MicrophonePermission] Request failed', e);
    return 'denied';
  }
}

