import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

const CAPTURE_INTERVAL_MS = 5000;
const JPEG_MAX_EDGE = 320;

// A Sessão de câmera vive com o processo; não para no unmount do React (hide / StrictMode).
let sessionStream: MediaStream | null = null;

export type CameraSession = {
  stream: MediaStream | null;
  cameraOk: boolean;
  cardVisible: boolean;
};

type Options = {
  onTick?: () => void;
};

export function useCameraSession(options: Options = {}): CameraSession {
  const { onTick } = options;
  const onTickRef = useRef(onTick);
  onTickRef.current = onTick;
  const [stream, setStream] = useState<MediaStream | null>(sessionStream);
  const [cameraOk, setCameraOk] = useState(sessionStream !== null);
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
        setCameraOk(ok);
      }
      onTickRef.current?.();
    };

    if (sessionStream) {
      setStream(sessionStream);
      void reportCamera(true);
      return () => {
        cancelled = true;
      };
    }

    if (!navigator.mediaDevices?.getUserMedia) {
      void reportCamera(false);
      return () => {
        cancelled = true;
      };
    }

    navigator.mediaDevices
      .getUserMedia({ video: true, audio: false })
      .then((media) => {
        if (!sessionStream) {
          sessionStream = media;
        } else if (sessionStream !== media) {
          media.getTracks().forEach((track) => track.stop());
        }
        if (cancelled) {
          return;
        }
        setStream(sessionStream);
        void reportCamera(true);
      })
      .catch(() => {
        void reportCamera(false);
      });

    return () => {
      cancelled = true;
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

  return { stream, cameraOk, cardVisible };
}

async function grabFrameJpeg(track: MediaStreamTrack): Promise<Uint8Array> {
  if (typeof ImageCapture === "function") {
    try {
      return await grabViaImageCapture(track);
    } catch {
      // WebView sem ImageCapture estável: um decode pontual via canvas.
    }
  }
  return grabViaOffscreenVideo(track);
}

async function grabViaImageCapture(track: MediaStreamTrack): Promise<Uint8Array> {
  const capture = new ImageCapture(track);
  const bitmap = await capture.grabFrame();
  try {
    return await bitmapToJpeg(bitmap);
  } finally {
    bitmap.close();
  }
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

async function bitmapToJpeg(bitmap: ImageBitmap): Promise<Uint8Array> {
  const canvas = document.createElement("canvas");
  sizeCanvas(canvas, bitmap.width, bitmap.height);
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("canvas");
  }
  ctx.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  const blob = await new Promise<Blob>((resolve, reject) => {
    canvas.toBlob(
      (next) => (next ? resolve(next) : reject(new Error("jpeg"))),
      "image/jpeg",
      0.5,
    );
  });
  return new Uint8Array(await blob.arrayBuffer());
}

function sizeCanvas(canvas: HTMLCanvasElement, width: number, height: number) {
  const edge = Math.max(width, height, 1);
  const scale = Math.min(1, JPEG_MAX_EDGE / edge);
  canvas.width = Math.max(1, Math.round(width * scale));
  canvas.height = Math.max(1, Math.round(height * scale));
}
