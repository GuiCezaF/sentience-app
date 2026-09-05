import appLogo from "../assets/logo.svg";
import type { TrayStatus } from "../types/agent";

const STATUS_CLASS: Record<TrayStatus, string> = {
  Ativo: "text-app-success",
  "Sem recorte": "text-app-text/55",
  Falha: "text-app-err",
};

type NavbarProps = {
  subjectName: string;
  status: TrayStatus;
};

export default function Navbar({ subjectName, status }: NavbarProps) {
  return (
    <nav className="flex items-center justify-between w-full text-sm">
      <div className="flex items-center gap-1.5">
        <img src={appLogo} className="w-18 h-9" alt="sentience logo" />
        <h1 className="text-sm font-semibold">{subjectName}</h1>
      </div>
      <div className={STATUS_CLASS[status]}>{status}</div>
    </nav>
  );
}
