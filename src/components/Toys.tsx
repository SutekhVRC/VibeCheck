import { Accordion, Badge } from "react-bootstrap";
import { useToys } from "../context/ToysContext";
import { percent } from "../utils";
import NameBadge from "./NameBadge";
import ToyFeatureForm from "./ToyFeatureForm";
import "./Toys.css";

export default function () {
  const { toys } = useToys();

  return (
    <div className="toys-container">
      <h1 className="grad-text">Connected toys</h1>
      {Object.keys(toys).length == 0 ? (
        <div>None</div>
      ) : (
        Object.values(toys).map((toy) => (
          <div className="toy-container" key={toy.toy_id}>
            <div className="toy">
              <NameBadge name={toy.toy_name} />
              <div
                style={{
                  color: `hsl(${toy.battery_level * 120}, 100%, 50%)`,
                }}
              >
                {percent.format(toy.battery_level)}
              </div>
            </div>
            <Accordion>
              {toy.features.map((feature) => (
                <Accordion.Item
                  eventKey={feature.feature_index.toString()}
                  key={feature.feature_index}
                >
                  <Accordion.Header>
                    <Badge
                      bg={feature.feature_enabled ? "success" : "danger"}
                    >{`${feature.feature_type} ${feature.feature_index}`}</Badge>
                  </Accordion.Header>
                  <Accordion.Body>
                    <ToyFeatureForm toyId={toy.toy_id} feature={feature} />
                  </Accordion.Body>
                </Accordion.Item>
              ))}
            </Accordion>
          </div>
        ))
      )}
    </div>
  );
}
