import React from "react";
import ReactDOM from "react-dom/client";
import { Toaster } from "sonner";
import App from "./App";
import { ThemeProvider } from "./components/theme-provider";
import { CoreEventProvider } from "./context/CoreEvents";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <CoreEventProvider>
        <App />
      </CoreEventProvider>
      <Toaster richColors />
    </ThemeProvider>
  </React.StrictMode>,
);
