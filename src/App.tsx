import { ToysProvider } from "./context/ToysContext";
import Toys from "./components/Toys/Toys";
import BottomButtons from "./components/BottomButtons";
import { CoreEventProvider } from "./context/CoreEventContext";
import { Footer } from "./components/Footer";
import "./App.css";

export default function App() {
  return (
    <CoreEventProvider>
      <h1>
        <div className="grad-forewards text-clip text-8xl">VibeCheck</div>
      </h1>
      <ToysProvider>
        <Toys />
      </ToysProvider>
      <BottomButtons />
      <Footer />
    </CoreEventProvider>
  );
}
