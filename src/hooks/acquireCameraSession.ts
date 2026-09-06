export type MediaDevicesLike = {
  getUserMedia(constraints: MediaStreamConstraints): Promise<MediaStream>;
};

type SessionSlot = {
  stream: MediaStream | null;
  inFlight: Promise<MediaStream> | null;
};

const SESSION_KEY = "__sentienceCameraSession";

function slot(): SessionSlot {
  const global = globalThis as typeof globalThis & { [SESSION_KEY]?: SessionSlot };
  global[SESSION_KEY] ??= { stream: null, inFlight: null };
  return global[SESSION_KEY];
}

export function peekCameraSession(): MediaStream | null {
  return slot().stream;
}

export function resetCameraSessionForTests(): void {
  const current = slot();
  current.stream = null;
  current.inFlight = null;
}

export function acquireCameraSession(
  devices: MediaDevicesLike | undefined = globalThis.navigator?.mediaDevices,
): Promise<MediaStream> {
  const current = slot();
  if (current.stream) {
    return Promise.resolve(current.stream);
  }
  if (current.inFlight) {
    return current.inFlight;
  }
  if (!devices?.getUserMedia) {
    return Promise.reject(new Error("getUserMedia"));
  }

  current.inFlight = devices
    .getUserMedia({ video: true, audio: false })
    .then((media) => {
      if (!current.stream) {
        current.stream = media;
      } else if (current.stream !== media) {
        media.getTracks().forEach((track) => track.stop());
      }
      return current.stream;
    })
    .finally(() => {
      current.inFlight = null;
    });

  return current.inFlight;
}
