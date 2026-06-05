import { FaCheck } from "react-icons/fa";
import { MdClose } from "react-icons/md";
import { invoke } from "@tauri-apps/api/core";
import { exit } from "@tauri-apps/plugin-process";

async function closeApp() {
  await exit(0);
}
async function logout(): Promise<void> {
  let logout = await invoke("logout");
  console.log(logout);
}

export default function Footer() {
  return (
    <footer className="flex flex-col items-start text-lg gap-2">
      <div className="flex gap-2 items-center justify-center">
        <FaCheck size={14} />
        <button onClick={() => logout()} className="cursor-pointer">
          Sair da conta
        </button>
      </div>
      <div className="flex gap-2 items-center justify-center text-app-err">
        <MdClose size={18} />
        <button onClick={() => closeApp()} className="cursor-pointer">
          Fechar app
        </button>
      </div>
    </footer>
  );
}
