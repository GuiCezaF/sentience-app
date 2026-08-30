import "./App.css";
import CameraFeed from "./components/CameraFeed";
import Footer from "./components/Footer";
import Navbar from "./components/Navbar";

export default function App() {
  return (
    <div className="min-h-screen flex flex-col p-4 gap-3">
      <Navbar />

      <main className="flex-1 flex flex-col items-center justify-center gap-2">
        <CameraFeed />
      </main>

      <Footer />
    </div>
  );
}
