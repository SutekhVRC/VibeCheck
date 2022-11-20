import { ReactNode } from "react";
import ReactDOM from "react-dom";
import "./Modal.css";

type modalProps = {
  isOpen: boolean;
  children: ReactNode;
  onClose: () => void;
};

export default function ({ isOpen, children, onClose }: modalProps) {
  if (!isOpen) {
    return null;
  }

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
