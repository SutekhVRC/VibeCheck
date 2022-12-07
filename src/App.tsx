import { ToysProvider } from "./context/ToysContext";
import Toys from "./components/Toys/Toys";
import BottomButtons from "./components/BottomButtons";
import { CoreEventProvider } from "./context/CoreEventContext";
import { Footer } from "./components/Footer";
import "./App.css";

export default function App() {
  return (
    <CoreEventProvider>
      <div className="flex-col gap-4">
        <div className="flex-col bg-zinc-800 rounded-md p-4">
          <h1>
            <div className="grad-text grad-forewards text-8xl">VibeCheck</div>
          </h1>
          <ToysProvider>
            <Toys />
          </ToysProvider>
        </div>
        <BottomButtons />
      </div>
      <Footer />
    </CoreEventProvider>
  );
}
