/**
 * Microphone Service
 * Manages audio input device enumeration and selection.
 */

import { log, logError } from '$lib/utils/logger';
import {
  checkMicrophonePermission,
  requestMicrophonePermission,
  type PermissionState,
} from './microphone-permission';

export interface AudioDevice {
  deviceId: string;
  label: string;
  isDefault?: boolean;
}

/** Result of device enumeration with permission status */
export interface DeviceEnumerationResult {
  devices: AudioDevice[];
  permissionState: PermissionState;
  error?: string;
}

/**
 * Get list of available audio input devices with permission handling
 *
 * This function checks permission state first, then enumerates devices.
 * If permission is 'prompt', it will NOT automatically request - call requestAndEnumerateDevices instead.
 */
export async function getAudioInputDevices(): Promise<DeviceEnumerationResult> {
  try {
    // Check current permission state
    const permissionStatus = await checkMicrophonePermission();

    if (!permissionStatus.isSupported) {
      return {
        devices: [],
        permissionState: 'denied',
        error: 'Audio recording is not supported on this device',
      };
    }

    // If permission not granted, return empty with state
    if (permissionStatus.state !== 'granted') {
      await log(`[Microphone] Permission state: ${permissionStatus.state}`);
      return {
        devices: [],
        permissionState: permissionStatus.state,
      };
    }

    // Permission granted - enumerate devices
    const devices = await navigator.mediaDevices.enumerateDevices();
    const audioDevices = devices
      .filter(device => device.kind === 'audioinput')
      .map((device, index) => ({
        deviceId: device.deviceId,
        label: device.label || `Microphone ${index + 1}`,
        isDefault: device.deviceId === 'default' || index === 0,
      }));

    await log(`[Microphone] Found ${audioDevices.length} audio devices`);

    return {
      devices: audioDevices,
      permissionState: 'granted',
    };
  } catch (e) {
    await logError('[Microphone] Failed to enumerate devices', e);
    return {
      devices: [],
      permissionState: 'unknown',
      error: e instanceof Error ? e.message : 'Failed to enumerate devices',
    };
  }
}

/**
 * Request permission and enumerate devices
 * Use this when user explicitly clicks "Allow Microphone"
 */
export async function requestAndEnumerateDevices(): Promise<DeviceEnumerationResult> {
  try {
    // Request permission (shows browser prompt)
    const permissionResult = await requestMicrophonePermission();

    if (permissionResult !== 'granted') {
      return {
        devices: [],
        permissionState: permissionResult,
      };
    }

    // Now enumerate devices
    return await getAudioInputDevices();
  } catch (e) {
    await logError('[Microphone] Failed to request and enumerate', e);
    return {
      devices: [],
      permissionState: 'denied',
      error: e instanceof Error ? e.message : 'Failed to get microphone access',
    };
  }
}

