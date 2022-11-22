import { useRef } from "react";
import Modal from "react-bootstrap/Modal";
import Form from "react-bootstrap/Form";
import "./Settings.css";
import { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import Button from "react-bootstrap/Button";
import InputGroup from "react-bootstrap/InputGroup";

type settingsModalProps = {
  settings: FeVibeCheckConfig | null; // Nullable because idk what to do if no response from b/e
  show: boolean;
  onHide: () => void;
  onSave: () => void;
};

export default function (props: settingsModalProps) {
  const oscBind = useRef(null);

  if (props.settings === null) {
    return (
      <Modal show={props.show} onHide={props.onHide} centered>
        <Modal.Header closeButton>
          <Modal.Title id="contained-modal-title-vcenter">Settings</Modal.Title>
        </Modal.Header>
        <Modal.Body>Settings could not be loaded!</Modal.Body>
      </Modal>
    );
  } else {
    return (
      <Modal show={props.show} onHide={props.onHide} centered>
        <Modal.Header closeButton>
          <Modal.Title id="contained-modal-title-vcenter">Settings</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          <Form
            onSubmit={(e) => {
              e.preventDefault();
              props.onSave();
              props.onHide();
            }}
          >
            <Form.Group className="setting">
              <Form.Label>OSC Bind</Form.Label>
              <InputGroup>
                <InputGroup.Text>IP:Port</InputGroup.Text>
                <Form.Control
                  autoFocus
                  defaultValue={props.settings.networking.bind}
                  ref={oscBind}
                  pattern={String.raw`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}:\d{1,5}$`}
                />
              </InputGroup>
            </Form.Group>
            <Button type="submit">Save</Button>
          </Form>
        </Modal.Body>
      </Modal>
    );
  }
}
