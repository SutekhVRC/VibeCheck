import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { Toaster } from "./components/ui/sonner";
import { CoreEventProvider } from "./context/CoreEvents";
import "./index.css";
import { ThemeProvider } from "./theme/ThemeProvider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <CoreEventProvider>
        <App />
      </CoreEventProvider>
      <Toaster />
    </ThemeProvider>
  </React.StrictMode>,
);
