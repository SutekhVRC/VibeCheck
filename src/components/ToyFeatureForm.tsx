import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import Form from "react-bootstrap/Form";
import { FeVCToyFeature } from "../../src-tauri/bindings/FeVCToyFeature";
import { useToys } from "../context/ToysContext";
import { ALTER_TOY, ALTER_TOY_DEBOUNCE } from "../data/constants";
import { round0 } from "../utils";
import "./ToyFeatureForm.css";

type ToyFeatureFormProps = {
  toyId: number;
  feature: FeVCToyFeature;
};

export default function ({ toyId, feature }: ToyFeatureFormProps) {
  const [modifiedFeature, setModifiedFeature] =
    useState<FeVCToyFeature>(feature);

  useEffect(() => {
    if (modifiedFeature == feature) return;
    async function setFeature() {
      await invoke(ALTER_TOY, {
        toyId: toyId,
        mutate: { Feature: modifiedFeature },
      });
    }
    const t = setTimeout(() => {
      setFeature();
    }, ALTER_TOY_DEBOUNCE);
    return () => clearTimeout(t);
  }, [modifiedFeature]);

  return (
    <>
      <div className="item">
        <Form.Label>Enabled</Form.Label>
        <Form.Check
          checked={modifiedFeature.feature_enabled}
          onChange={(e) =>
            setModifiedFeature((s) => {
              return { ...s, feature_enabled: e.target.checked };
            })
          }
        />
      </div>
      <div className="item">
        <Form.Label>OSC Parameter</Form.Label>
        <div></div>
        <Form.Control
          value={modifiedFeature.osc_parameter}
          onChange={(e) =>
            setModifiedFeature((s) => {
              return { ...s, osc_parameter: e.target.value };
            })
          }
        />
      </div>
      <div className="item">
        <Form.Label>Smoothing</Form.Label>
        <Form.Check
          checked={modifiedFeature.smooth_enabled}
          onChange={(e) =>
            setModifiedFeature((s) => {
              return { ...s, smooth_enabled: e.target.checked };
            })
          }
        />
        <Form.Range
          disabled={!modifiedFeature.smooth_enabled}
          min={1}
          max={20}
          step={1}
          value={modifiedFeature.feature_levels.smooth_rate}
          onChange={(e) =>
            setModifiedFeature((s) => {
              return {
                ...s,
                feature_levels: {
                  ...modifiedFeature.feature_levels,
                  smooth_rate: Number(e.target.value),
                },
              };
            })
          }
        />
        {modifiedFeature.feature_levels.smooth_rate}
      </div>
      <div className="item">
        <Form.Label>Idle</Form.Label>
        <div></div>
        <Form.Range
          min={0}
          max={1}
          step={0.01}
          value={modifiedFeature.feature_levels.idle_level}
          onChange={(e) =>
            setModifiedFeature((s) => {
              return {
                ...s,
                feature_levels: {
                  ...modifiedFeature.feature_levels,
                  idle_level: Number(e.target.value),
                },
              };
            })
          }
        />
        {round0.format(modifiedFeature.feature_levels.idle_level * 100)}
      </div>
      <div className="item">
        <Form.Label>Minimum</Form.Label>
        <div></div>
        <Form.Range
          min={0}
          max={1}
          step={0.01}
          value={modifiedFeature.feature_levels.minimum_level}
          onChange={(e) =>
            setModifiedFeature((s) => {
              return {
                ...s,
                feature_levels: {
                  ...modifiedFeature.feature_levels,
                  minimum_level: Number(e.target.value),
                },
              };
            })
          }
        />
        {round0.format(modifiedFeature.feature_levels.minimum_level * 100)}
      </div>
      <div className="item">
        <Form.Label>Maximum</Form.Label>
        <div></div>
        <Form.Range
          min={0}
          max={1}
          step={0.01}
          value={modifiedFeature.feature_levels.maximum_level}
          onChange={(e) =>
            setModifiedFeature((s) => {
              return {
                ...s,
                feature_levels: {
                  ...modifiedFeature.feature_levels,
                  maximum_level: Number(e.target.value),
                },
              };
            })
          }
        />
        {round0.format(modifiedFeature.feature_levels.maximum_level * 100)}
      </div>
    </>
  );
}
