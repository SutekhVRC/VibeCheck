import { useState } from "react";
import Form from "react-bootstrap/Form";
import { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import { round2 } from "../utils";
import "./ToySettings.css";

export default function (props: FeVCToyFeature) {
  const [state, setState] = useState<FeVCToyFeature>(props);

  return (
    <>
      <div className="item">
        <Form.Label>Enabled</Form.Label>
        <Form.Check
          checked={state.feature_enabled}
          onChange={(e) =>
            setState((s) => {
              return { ...s, feature_enabled: e.target.checked };
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
          value={state.osc_parameter}
          onChange={(e) =>
            setState((s) => {
              return { ...s, osc_parameter: e.target.value };
            })
          }
        />
        <div />
      </div>
      <div className="item">
        <Form.Label>Smoothing</Form.Label>
        <Form.Check
          checked={state.smooth_enabled}
          onChange={(e) =>
            setState((s) => {
              return { ...s, smooth_enabled: e.target.checked };
            })
          }
        />
        <Form.Range
          disabled={!state.smooth_enabled}
          min={1}
          max={20}
          step={1}
          value={state.feature_levels.smooth_rate}
          onChange={(e) =>
            setState((s) => {
              return {
                ...s,
                feature_levels: {
                  ...state.feature_levels,
                  smooth_rate: Number(e.target.value),
                },
              };
            })
          }
        />
        {state.feature_levels.smooth_rate}
      </div>
      <div className="item">
        <Form.Label>Idle</Form.Label>
        <div />
        <Form.Range
          min={0}
          max={1}
          step={0.01}
          value={state.feature_levels.idle_level}
          onChange={(e) =>
            setState((s) => {
              return {
                ...s,
                feature_levels: {
                  ...state.feature_levels,
                  idle_level: Number(e.target.value),
                },
              };
            })
          }
        />
        {round2.format(state.feature_levels.idle_level)}
      </div>
      <div className="item">
        <Form.Label>Minimum</Form.Label>
        <div />
        <Form.Range
          min={0}
          max={1}
          step={0.01}
          value={state.feature_levels.minimum_level}
          onChange={(e) =>
            setState((s) => {
              return {
                ...s,
                feature_levels: {
                  ...state.feature_levels,
                  minimum_level: Number(e.target.value),
                },
              };
            })
          }
        />
        {round2.format(state.feature_levels.minimum_level)}
      </div>
      <div className="item">
        <Form.Label>Maximum</Form.Label>
        <div />
        <Form.Range
          min={0}
          max={1}
          step={0.01}
          value={state.feature_levels.maximum_level}
          onChange={(e) =>
            setState((s) => {
              return {
                ...s,
                feature_levels: {
                  ...state.feature_levels,
                  maximum_level: Number(e.target.value),
                },
              };
            })
          }
        />
        {round2.format(state.feature_levels.maximum_level)}
      </div>
    </>
  );
}
