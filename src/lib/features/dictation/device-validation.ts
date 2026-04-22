/**
 * Microphone Device Validation
 *
 * Shared device validation used by AudioRecorder and MicrophoneTest.
 */

import { log } from '$lib/utils/logger';

/**
 * Validate that a microphone device exists and is available.
 *
 * @param deviceId - The device ID to validate
 * @returns The matching MediaDeviceInfo
 * @throws Error if the device is not found or no microphones are available
 */
export async function validateMicrophoneDevice(deviceId: string): Promise<MediaDeviceInfo> {
  const devices = await navigator.mediaDevices.enumerateDevices();
  const audioInputs = devices.filter(d => d.kind === 'audioinput');
  const target = audioInputs.find(d => d.deviceId === deviceId);

  if (!target) {
    if (audioInputs.length === 0) {
      throw new Error('No microphones found. Please connect a microphone.');
    }
    throw new Error(
      'Selected microphone not found. It may have been disconnected. ' +
      'Please select a different microphone in Dictation settings.'
    );
  }

  await log(`[DeviceValidation] Validated device: ${target.label || target.deviceId.slice(0, 8)}`);
  return target;
}
