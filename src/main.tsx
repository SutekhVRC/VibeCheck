import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ToastProvider } from "./components/Toast";
import { CoreEventProvider } from "./context/CoreEventContext";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ToastProvider>
      <CoreEventProvider>
        <App />
      </CoreEventProvider>
    </ToastProvider>
  </React.StrictMode>
);
