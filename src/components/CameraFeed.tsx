import { useEffect, useRef } from "react";
import noCamIcon from "../assets/no-cam.svg";

type CameraFeedProps = {
  stream: MediaStream | null;
  visible: boolean;
  cameraOk: boolean;
};

export default function CameraFeed({ stream, visible, cameraOk }: CameraFeedProps) {
  const videoRef = useRef<HTMLVideoElement | null>(null);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) {
      return;
    }
    video.srcObject = visible && cameraOk ? stream : null;
  }, [stream, visible, cameraOk]);

  return (
    <div className="w-full aspect-video rounded-app overflow-hidden flex items-center justify-center ring-1 ring-app-text/10 bg-app-surface">
      {cameraOk ? (
        visible ? (
          <video
            ref={videoRef}
            autoPlay
            playsInline
            muted
            className="w-full h-full object-cover scale-x-[-1]"
          />
        ) : null
      ) : (
        <img src={noCamIcon} alt="Câmera indisponível" className="w-12 h-12 opacity-60" />
      )}
    </div>
  );
}
