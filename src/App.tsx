import "./App.css";
import CameraFeed from "./components/CameraFeed";
import Footer from "./components/Footer";
import Navbar from "./components/Navbar";
import { useAgentSnapshot } from "./hooks/useAgentSnapshot";
import { useCameraSession } from "./hooks/useCameraSession";

const SUBJECT_PLACEHOLDER = "Guilherme";

export default function App() {
  const { snapshot, refresh } = useAgentSnapshot();
  const { stream, cameraOk, cardVisible } = useCameraSession({ onTick: refresh });

  return (
    <div className="min-h-screen flex flex-col p-4 gap-3">
      <Navbar subjectName={SUBJECT_PLACEHOLDER} status={snapshot.status} />

      <main className="flex-1 flex flex-col justify-center gap-2">
        <CameraFeed stream={stream} visible={cardVisible} cameraOk={cameraOk} />
        <span className="text-xs text-app-text/50">Sincronização {snapshot.sync}</span>
      </main>

      <Footer />
    </div>
  );
}
