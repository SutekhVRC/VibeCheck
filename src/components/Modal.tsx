import { ReactNode } from "react";
import ReactDOM from "react-dom";
import Button from "react-bootstrap/Button";
import "./Modal.css";

type modalProps = {
  children: ReactNode;
  onClose: () => void;
};

export default function ({ children, onClose }: modalProps) {
  return ReactDOM.createPortal(
    <>
      <div className="modal-overlay" onClick={onClose} />
      <div className="modal-custom">
        {children}
        <br />
        <div className="modal-buttons">
          <Button variant="light" onClick={onClose}>
            Close
          </Button>
          <Button variant="light" onClick={onClose}>
            Save
          </Button>
        </div>
      </div>
    </>,
    document.getElementById("portal")!
  );
}
