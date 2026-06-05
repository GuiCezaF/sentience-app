import "./App.css";
import Footer from "./components/Footer";
import Navbar from "./components/Navbar";

export default function App() {
  return (
    <div className="min-h-screen flex flex-col p-4 gap-3">
      <Navbar />

      <main className="flex-1">Conteudo do app</main>

      <Footer />
    </div>
  );
}
