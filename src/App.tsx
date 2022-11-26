import { ToysProvider } from "./context/ToysContext";
import Header from "./components/Header";
import Toys from "./components/Toys";
import Footer from "./components/Footer";
import "./App.css";
import { CoreEventProvider } from "./context/CoreEventContext";

export default function App() {
  return (
    <div style={{ display: "flex", justifyContent: "center" }}>
      <div className="main-container">
        <Header />
        <ToysProvider>
          <Toys />
        </ToysProvider>
        <CoreEventProvider>
          <Footer />
        </CoreEventProvider>
      </div>
    </div>
  );
}
