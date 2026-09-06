import { useEffect, useRef, useState, type ReactNode } from "react";
import noCamIcon from "../assets/no-cam.svg";
import type { CameraFailure, CameraState } from "../hooks/useCameraSession";

type CameraFeedProps = {
  stream: MediaStream | null;
  state: CameraState;
  failure: CameraFailure | null;
  visible: boolean;
  children?: ReactNode;
};

function failureHint(failure: CameraFailure | null): string {
  switch (failure?.kind) {
    case "denied":
      return "Permissão de câmera negada para o Sentience.";
    case "missing":
      return "Nenhuma câmera encontrada neste computador.";
    case "busy":
      return "A câmera está em uso por outro aplicativo.";
    case "unknown":
      return `Erro ao abrir a câmera (${failure.name}).`;
    default:
      return "A Captura retoma assim que a câmera voltar.";
  }
}

export default function CameraFeed({
  stream,
  state,
  failure,
  visible,
  children,
}: CameraFeedProps) {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const [ready, setReady] = useState(false);
  const showVideo = state === "ok" && visible;

  useEffect(() => {
    const video = videoRef.current;
    if (!video) {
      return;
    }
    if (showVideo && stream) {
      video.srcObject = stream;
      void video.play().catch(() => {
      });
      return;
    }
    video.srcObject = null;
    setReady(false);
  }, [stream, showVideo]);

  return (
    <section className="camera" data-state={state} aria-label="Câmera">
      {showVideo ? (
        <video
          ref={videoRef}
          autoPlay
          playsInline
          muted
          className="camera-video"
          data-ready={ready}
          onPlaying={() => setReady(true)}
        />
      ) : null}

      {state === "requesting" || (showVideo && !ready) ? (
        <div className="camera-empty" role="status">
          <span className="camera-connecting-dot" aria-hidden="true" />
          <p className="camera-empty-title">Conectando à câmera…</p>
        </div>
      ) : null}

      {state === "unavailable" ? (
        <div className="camera-empty" role="status">
          <img src={noCamIcon} alt="" className="camera-empty-icon" />
          <p className="camera-empty-title">Câmera indisponível</p>
          <p className="camera-empty-hint">{failureHint(failure)}</p>
          <p className="camera-empty-hint">Tentando novamente a cada 5 s.</p>
        </div>
      ) : null}

      {children}
    </section>
  );
}
