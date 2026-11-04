import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "./components/theme-provider";
import { Toaster } from "./components/ui/sonner";
import { CoreEventProvider } from "./context/CoreEvents";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <CoreEventProvider>
        <App />
      </CoreEventProvider>
      <Toaster />
    </ThemeProvider>
  </React.StrictMode>,
);
