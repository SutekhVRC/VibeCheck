import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { Toaster } from "./components/ui/sonner";
import { AvatarCatalogProvider } from "./context/AvatarCatalog";
import { CoreEventProvider } from "./context/CoreEvents";
import "./index.css";
import { ThemeProvider } from "./theme/ThemeProvider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {/* The theme comes from the config, so the config provider goes first */}
    <CoreEventProvider>
      <ThemeProvider>
        <AvatarCatalogProvider>
          <App />
        </AvatarCatalogProvider>
        <Toaster />
      </ThemeProvider>
    </CoreEventProvider>
  </React.StrictMode>,
);
