import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { acquireCameraSession, peekCameraSession } from "./acquireCameraSession";

const CAPTURE_INTERVAL_MS = 5000;
const JPEG_MAX_EDGE = 320;

export type CameraState = "requesting" | "ok" | "unavailable";

export type CameraFailure =
  | { kind: "denied" }
  | { kind: "missing" }
  | { kind: "busy" }
  | { kind: "unknown"; name: string };

export type CameraSession = {
  stream: MediaStream | null;
  cameraState: CameraState;
  cameraFailure: CameraFailure | null;
  cardVisible: boolean;
};

const RETRY_INTERVAL_MS = 5000;

export function classifyCameraError(error: unknown): CameraFailure {
  const name = error instanceof Error ? error.name : String(error);
  switch (name) {
    case "NotAllowedError":
    case "PermissionDeniedError":
    case "SecurityError":
      return { kind: "denied" };
    case "NotFoundError":
    case "DevicesNotFoundError":
      return { kind: "missing" };
    case "NotReadableError":
    case "TrackStartError":
    case "SourceUnavailableError":
    case "AbortError":
      return { kind: "busy" };
    default:
      return { kind: "unknown", name };
  }
}

type Options = {
  onTick?: () => void;
};

export function useCameraSession(options: Options = {}): CameraSession {
  const { onTick } = options;
  const onTickRef = useRef(onTick);
  onTickRef.current = onTick;
  const [stream, setStream] = useState<MediaStream | null>(peekCameraSession());
  const [cameraState, setCameraState] = useState<CameraState>(
    peekCameraSession() ? "ok" : "requesting",
  );
  const [cameraFailure, setCameraFailure] = useState<CameraFailure | null>(null);
  const [cardVisible, setCardVisible] = useState(false);

  useEffect(() => {
    let cancelled = false;

    const syncVisibility = async () => {
      try {
        const visible = await getCurrentWindow().isVisible();
        if (!cancelled) {
          setCardVisible(visible);
        }
      } catch {
        if (!cancelled) {
          setCardVisible(true);
        }
      }
    };

    void syncVisibility();

    let unlisten: (() => void) | undefined;
    try {
      getCurrentWindow()
        .onFocusChanged(({ payload: focused }) => {
          if (cancelled) {
            return;
          }
          if (!focused) {
            setCardVisible(false);
            return;
          }
          void syncVisibility();
        })
        .then((fn) => {
          if (cancelled) {
            fn();
            return;
          }
          unlisten = fn;
        })
        .catch(() => {
          if (!cancelled) {
            setCardVisible(true);
          }
        });
    } catch {
      if (!cancelled) {
        setCardVisible(true);
      }
    }

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let cancelled = false;

    const reportCamera = async (ok: boolean) => {
      try {
        await invoke("set_camera_ok", { ok });
      } catch {
        // Mesmo fallback do snapshot
      }
      if (!cancelled) {
        setCameraState(ok ? "ok" : "unavailable");
      }
      onTickRef.current?.();
    };

    let retry: number | undefined;

    const attempt = () => {
      void acquireCameraSession()
        .then((media) => {
          if (cancelled) {
            return;
          }
          setStream(media);
          setCameraFailure(null);
          void reportCamera(true);
        })
        .catch((error: unknown) => {
          if (cancelled) {
            return;
          }
          const existing = peekCameraSession();
          if (existing) {
            setStream(existing);
            setCameraFailure(null);
            void reportCamera(true);
            return;
          }
          setCameraFailure(classifyCameraError(error));
          void reportCamera(false);
          retry = window.setTimeout(attempt, RETRY_INTERVAL_MS);
        });
    };

    attempt();

    return () => {
      cancelled = true;
      window.clearTimeout(retry);
    };
  }, []);

  useEffect(() => {
    if (!stream) {
      return;
    }

    let cancelled = false;

    const tick = async () => {
      const track = stream.getVideoTracks()[0];
      if (!track || cancelled) {
        return;
      }
      try {
        const bytes = await grabFrameJpeg(track);
        if (cancelled) {
          return;
        }
        await invoke("ingest_frame", { bytes: Array.from(bytes) });
        onTickRef.current?.();
      } catch {
        // Tick sem Captura;
      }
    };

    void tick();
    const id = window.setInterval(() => {
      void tick();
    }, CAPTURE_INTERVAL_MS);

    return () => {
      cancelled = true;
      window.clearInterval(id);
    };
  }, [stream]);

  return { stream, cameraState, cameraFailure, cardVisible };
}

async function grabFrameJpeg(track: MediaStreamTrack): Promise<Uint8Array> {
  return grabViaOffscreenVideo(track);
}

function grabViaOffscreenVideo(track: MediaStreamTrack): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const video = document.createElement("video");
    video.muted = true;
    video.playsInline = true;
    video.srcObject = new MediaStream([track]);

    const fail = (error: unknown) => {
      video.srcObject = null;
      reject(error);
    };

    video.onloadeddata = () => {
      const canvas = document.createElement("canvas");
      sizeCanvas(canvas, video.videoWidth, video.videoHeight);
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        fail(new Error("canvas"));
        return;
      }
      ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
      video.srcObject = null;
      canvas.toBlob(
        (blob) => {
          if (!blob) {
            fail(new Error("jpeg"));
            return;
          }
          void blob.arrayBuffer().then((buf) => resolve(new Uint8Array(buf)), fail);
        },
        "image/jpeg",
        0.5,
      );
    };
    video.play().catch(fail);
  });
}

function sizeCanvas(canvas: HTMLCanvasElement, width: number, height: number) {
  const edge = Math.max(width, height, 1);
  const scale = Math.min(1, JPEG_MAX_EDGE / edge);
  canvas.width = Math.max(1, Math.round(width * scale));
  canvas.height = Math.max(1, Math.round(height * scale));
}
