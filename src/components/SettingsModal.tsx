import { useEffect, useRef, useState } from "react";
import Button from "react-bootstrap/Button";
import InputGroup from "react-bootstrap/InputGroup";
import { invoke } from "@tauri-apps/api";
import Modal from "react-bootstrap/Modal";
import Form from "react-bootstrap/Form";

import { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import { GET_CONFIG, SET_CONFIG } from "../data/constants";
import "./SettingsModal.css";

type settingsModalProps = {
  show: boolean;
  onHide: () => void;
};

export default function (props: settingsModalProps) {
  const [config, setConfig] = useState<null | FeVibeCheckConfig>(null);
  const oscBind = useRef<HTMLInputElement>(null);

  useEffect(() => {
    async function getConfig() {
      return await invoke<FeVibeCheckConfig>(GET_CONFIG);
    }
    getConfig()
      .then((r) => setConfig(r))
      .catch(() => setConfig(null));
  });

  async function updateConfig() {
    if (oscBind.current?.value == null || config == null) {
      return;
    }
    const newConfig: FeVibeCheckConfig = {
      ...config,
      networking: {
        ...config.networking,
        bind: oscBind.current.value,
      },
    };
    await invoke(SET_CONFIG, { feVcConfig: newConfig });
  }

  return (
    <Modal {...props} centered>
      <Modal.Header closeButton>
        <Modal.Title id="contained-modal-title-vcenter">Settings</Modal.Title>
      </Modal.Header>
      {config == null ? (
        <Modal.Body>Could not load settings</Modal.Body>
      ) : (
        <Modal.Body>
          <Form
            onSubmit={(e) => {
              e.preventDefault();
              updateConfig();
              props.onHide();
            }}
          >
            <Form.Group className="setting">
              <Form.Label>OSC Bind</Form.Label>
              <InputGroup>
                <InputGroup.Text>IP:Port</InputGroup.Text>
                <Form.Control
                  autoFocus
                  defaultValue={config.networking.bind}
                  ref={oscBind}
                  pattern={String.raw`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}:\d{1,5}$`}
                />
              </InputGroup>
            </Form.Group>
            <Button type="submit">Save</Button>
          </Form>
        </Modal.Body>
      )}
    </Modal>
  );
}
