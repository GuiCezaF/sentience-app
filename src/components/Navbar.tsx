import type { TrayStatus } from "../types/agent";
import StatusPill from "./StatusPill";

const STATUS_HINT: Record<TrayStatus, string> = {
  Ativo: "Classificando expressões",
  "Sem recorte": "Nenhum rosto no recorte",
  Falha: "Câmera ou modelo indisponível",
};

type NavbarProps = {
  subjectName: string;
  status: TrayStatus;
};

export default function Navbar({ subjectName, status }: NavbarProps) {
  return (
    <header className="header">
      <div className="min-w-0">
        <h1 className="header-title">{subjectName}</h1>
        <p className="header-subtitle">{STATUS_HINT[status]}</p>
      </div>
      <StatusPill status={status} />
    </header>
  );
}
