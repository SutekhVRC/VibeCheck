import { ToysProvider } from "./context/ToysContext";
import Header from "./components/Header";
import Toys from "./components/Toys";
import Footer from "./components/Footer";
import "./App.css";

export default function App() {
  return (
    <div style={{ display: "flex", justifyContent: "center" }}>
      <div className="main-container">
        <ToysProvider>
          <Header />
          <Toys />
          <Footer />
        </ToysProvider>
      </div>
    </div>
  );
}
