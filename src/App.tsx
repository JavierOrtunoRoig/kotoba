import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import MainApp from "./MainApp";
import PopupApp from "./PopupApp";
import "./styles.css";

function currentWindowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return "main";
  }
}

export default function App() {
  const label = currentWindowLabel();
  const variant: "popup" | "widget" | null =
    label === "popup" ? "popup" : label === "widget" ? "widget" : null;

  useEffect(() => {
    document.body.classList.toggle("popup-window", variant !== null);
    document.body.classList.toggle("widget-window", label === "widget");
  }, [variant, label]);

  return variant ? <PopupApp variant={variant} /> : <MainApp />;
}
