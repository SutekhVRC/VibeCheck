import { useRef } from "react";
import Modal from "react-bootstrap/Modal";
import Form from "react-bootstrap/Form";
import "./Settings.css";
import { FeVibeCheckConfig } from "../../src-tauri/bindings/FeVibeCheckConfig";
import Button from "react-bootstrap/Button";
import InputGroup from "react-bootstrap/InputGroup";
import { invoke } from "@tauri-apps/api";
import { SET_CONFIG } from "../data/constants";

type settingsModalProps = {
  settings: FeVibeCheckConfig | null; // Nullable because backend sets default, don't want to overwrite if no response
  show: boolean;
  onHide: () => void;
  onSave: () => void;
};

// I think we need to do something like this:
// type InvokeableConfig = FeVibeCheckConfig & { [key: string]: unknown };
// Because InvokeArgs has type [key:string]:unkown, we need to union w/ the interface to be able to pass into invoke()

// Right now FeOSCNetworking is {bind:string, remote:string}
// So we also need to be careful to not modify the other attributes

// The signature of set_vibecheck_config is:
//    (vc_state: tauri::State<'_, vcore::VCStateMutex>, fe_vc_config: FeVibeCheckConfig) -> Result<(), frontend::VCFeError>
// So we need to also wrap the InvokeableType with feVcConfig
type InvokeableConfig = { feVcConfig: FeVibeCheckConfig } & {
  [key: string]: unknown;
};

export default function (props: settingsModalProps) {
  const oscBind = useRef<HTMLInputElement>(null);

  async function setConfig() {
    if (
      oscBind.current == null ||
      oscBind.current.value == null ||
      props.settings == null
    ) {
      return;
    }
    const newConfig: InvokeableConfig = {
      feVcConfig: {
        networking: {
          bind: oscBind.current.value,
          remote: props.settings.networking.remote,
        },
      },
    };
    await invoke(SET_CONFIG, newConfig);
  }

  if (props.settings == null) {
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
              setConfig();
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
