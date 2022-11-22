import { useRef } from "react";
import Form from "react-bootstrap/Form";
import "./Settings.css";

export type settingsProps = {
  defaultHost: string;
  defaultPort: string;
};

export default function ({ defaultHost, defaultPort }: settingsProps) {
  const host = useRef(null);
  const port = useRef(null);

  return (
    <>
      <h2>Settings</h2>
      <Form>
        <Form.Group className="setting">
          <Form.Label>OSC Bind Host</Form.Label>
          <Form.Control
            autoFocus
            defaultValue={defaultHost}
            ref={host}
            pattern={`^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$`}
          />
        </Form.Group>
        <Form.Group className="setting">
          <Form.Label>OSC Bind Port</Form.Label>
          <Form.Control
            defaultValue={defaultPort}
            ref={port}
            pattern={`\d{1,5}`}
          />
        </Form.Group>
      </Form>
    </>
  );
}
