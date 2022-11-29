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
  const scanOnDisconnect = useRef<HTMLInputElement>(null);
  const minimizeOnExit = useRef<HTMLInputElement>(null);
  const desktopNotifications = useRef<HTMLInputElement>(null);

  async function getConfig() {
    await invoke<FeVibeCheckConfig>(GET_CONFIG).then((r) => setConfig(r));
  }

  useEffect(() => {
    getConfig();
  }, []);

  async function updateConfig() {
    if (
      !config ||
      !oscBind.current ||
      !scanOnDisconnect.current ||
      !minimizeOnExit.current ||
      !desktopNotifications.current
    ) {
      return;
    }
    const newConfig: FeVibeCheckConfig = {
      ...config,
      scan_on_disconnect: scanOnDisconnect.current.checked,
      minimize_on_exit: minimizeOnExit.current.checked,
      desktop_notifications: desktopNotifications.current.checked,
      networking: {
        ...config.networking,
        bind: oscBind.current.value,
      },
    };
    console.log(newConfig);
    await invoke(SET_CONFIG, { feVcConfig: newConfig });
    getConfig();
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
            <div className="setting">
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
              <Form.Label>Scan on Disconnect</Form.Label>
              <Form.Check
                type="switch"
                defaultChecked={config.scan_on_disconnect}
                ref={scanOnDisconnect}
              />
              <Form.Label>Minimize on Exit</Form.Label>
              <Form.Check
                type="switch"
                defaultChecked={config.minimize_on_exit}
                ref={minimizeOnExit}
              />
              <Form.Label>Desktop Notifications</Form.Label>
              <Form.Check
                type="switch"
                defaultChecked={config.desktop_notifications}
                ref={desktopNotifications}
              />
            </div>
            <Button type="submit">Save</Button>
          </Form>
        </Modal.Body>
      )}
    </Modal>
  );
}
