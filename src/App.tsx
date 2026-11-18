import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import MainApp from "./MainApp";
import PopupApp from "./PopupApp";
import "./styles.css";

export function isPopupWindow(): boolean {
  try {
    return getCurrentWindow().label === "popup";
  } catch {
    return false;
  }
}

export default function App() {
  const popup = isPopupWindow();

  useEffect(() => {
    document.body.classList.toggle("popup-window", popup);
  }, [popup]);

  return popup ? <PopupApp /> : <MainApp />;
}
