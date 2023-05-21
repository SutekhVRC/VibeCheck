import { ToysProvider } from "./context/ToysContext";
import Toys from "./features/Toys";
import { CoreEventProvider } from "./context/CoreEventContext";
import { Footer } from "./features/Footer";
import VibecheckLogo from "./assets/VibeCheck_logo.png";
import {ToastProvider} from "./context/ToastContext"
import "./App.css";

export default function App() {
  return (
    <ToastProvider>
      <CoreEventProvider>
        <div className="flex justify-center h-20 mb-4">
          <img src={VibecheckLogo} />
        </div>
        <ToysProvider>
          <Toys />
        </ToysProvider>
        <Footer />
      </CoreEventProvider>
    </ToastProvider>
  );
}
