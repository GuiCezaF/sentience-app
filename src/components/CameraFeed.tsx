import { useEffect, useRef, useState } from "react";
import noCamIcon from "../assets/no-cam.svg";

export default function CameraFeed() {
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const [hasCamera, setHasCamera] = useState(true);
  const [emotion] = useState("Neutro");

  useEffect(() => {
    let stream: MediaStream | null = null;

    navigator.mediaDevices
      ?.getUserMedia({ video: true, audio: false })
      .then((mediaStream) => {
        stream = mediaStream;
        if (videoRef.current) {
          videoRef.current.srcObject = mediaStream;
        }
        setHasCamera(true);
      })
      .catch(() => {
        setHasCamera(false);
      });

    return () => {
      if (stream) {
        stream.getTracks().forEach((track) => track.stop());
      }
    };
  }, []);

  return (
    <>
      <div className="w-[140px] h-[140px] rounded-full overflow-hidden flex items-center justify-center ring-2 ring-app-text/10 bg-black/5">
        {hasCamera ? (
          <video
            ref={videoRef}
            autoPlay
            playsInline
            muted
            className="w-full h-full object-cover scale-x-[-1]"
          />
        ) : (
          <img src={noCamIcon} alt="Câmera indisponível" className="w-12 h-12 opacity-60" />
        )}
      </div>

      {hasCamera ? (
        <span className="text-sm font-semibold text-app-text">{emotion}</span>
      ) : (
        <span className="text-xs font-semibold text-app-err">Câmera indisponível</span>
      )}
    </>
  );
}
