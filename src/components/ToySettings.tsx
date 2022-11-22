import { useState } from "react";
import Form from "react-bootstrap/Form";
import { round2 } from "../utils";
import "./ToySettings.css";

type FeatureData = {
  enabled: boolean;
  oscParameter: string;
  smoothing: boolean;
  smoothingRate: number;
  idle: number;
  minimum: number;
  maximum: number;
};

const INITIAL_DATA: FeatureData = {
  enabled: true,
  oscParameter: "/avatar/parameters/vib",
  smoothing: true,
  smoothingRate: 2,
  idle: 0,
  minimum: 0,
  maximum: 1,
};

export default function () {
  const [state, setState] = useState<FeatureData>(INITIAL_DATA);

  return (
    <>
      <div className="item">
        <Form.Label>Enabled</Form.Label>
        <Form.Check
          checked={state.enabled}
          onChange={(e) =>
            setState((s) => {
              return { ...s, enabled: e.target.checked };
            })
          }
        />
        <div />
        <div />
      </div>
      <div className="item">
        <Form.Label>OSC Parameter</Form.Label>
        <div />
        <Form.Control
          value={state.oscParameter}
          onChange={(e) =>
            setState((s) => {
              return { ...s, oscParameter: e.target.value };
            })
          }
        />
        <div />
      </div>
      <div className="item">
        <Form.Label>Smoothing</Form.Label>
        <Form.Check
          checked={state.smoothing}
          onChange={(e) =>
            setState((s) => {
              return { ...s, smoothing: e.target.checked };
            })
          }
        />
        <Form.Range
          disabled={!state.smoothing}
          min={1}
          max={20}
          step={1}
          value={state.smoothingRate}
          onChange={(e) =>
            setState((s) => {
              return { ...s, smoothingRate: Number(e.target.value) };
            })
          }
        />
        {state.smoothingRate}
      </div>
      <div className="item">
        <Form.Label>Idle</Form.Label>
        <div />
        <Form.Range
          min={0}
          max={1}
          step={0.01}
          value={state.idle}
          onChange={(e) =>
            setState((s) => {
              return { ...s, idle: Number(e.target.value) };
            })
          }
        />
        {round2.format(state.idle)}
      </div>
      <div className="item">
        <Form.Label>Minimum</Form.Label>
        <div />
        <Form.Range
          min={0}
          max={1}
          step={0.01}
          value={state.minimum}
          onChange={(e) =>
            setState((s) => {
              return { ...s, minimum: Number(e.target.value) };
            })
          }
        />
        {round2.format(state.minimum)}
      </div>
      <div className="item">
        <Form.Label>Maximum</Form.Label>
        <div />
        <Form.Range
          min={0}
          max={1}
          step={0.01}
          value={state.maximum}
          onChange={(e) =>
            setState((s) => {
              return { ...s, maximum: Number(e.target.value) };
            })
          }
        />
        {round2.format(state.maximum)}
      </div>
    </>
  );
}
