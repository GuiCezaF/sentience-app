import { useState } from "react";
import appLogo from "../assets/logo.svg";

function getUserName(): string {
  // TODO: get real user's name
  return "Guilherme";
}
function getStatus(): string {
  // TODO: get real app status
  // Server status or emotion process status ?
  return "Inativo";
}

export default function Navbar() {
  const [userName, setUserStatus] = useState(getUserName());
  const [appStatus, setAppStatus] = useState(getStatus());

  const statusColors: Record<string, string> = {
    Ativo: "text-app-success",
    Inativo: "text-app-err",
    Maintenance: "text-app-warn",
  };

  return (
    <nav className="flex items-center justify-between w-full text-sm">
      <div className="flex items-center gap-1.5">
        <img src={appLogo} className="w-18 h-9" alt="sentience logo" />
        <h1 className="text-sm font-bold">{userName}</h1>
      </div>
      <div className={statusColors[appStatus] || "text-gray-500"}>
        {appStatus}
      </div>
    </nav>
  );
}
