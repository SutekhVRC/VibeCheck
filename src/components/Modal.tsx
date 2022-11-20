import { ReactNode } from "react";
import ReactDOM from "react-dom";
import "./Modal.css";

type modalProps = {
  children: ReactNode;
  onClose: () => void;
};

export default function ({ children, onClose }: modalProps) {
  return ReactDOM.createPortal(
    <>
      <div className="modal-overlay" onClick={onClose} />
      <div className="modal">
        {children}
        <br />
        <div className="modal-buttons">
          <button onClick={onClose}>Close</button>
          <button onClick={onClose}>Save</button>
        </div>
      </div>
    </>,
    document.getElementById("portal")!
  );
}
