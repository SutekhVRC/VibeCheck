import { Badge } from "react-bootstrap";
import Accordion from "react-bootstrap/esm/Accordion";
import { FeVCToy } from "../../src-tauri/bindings/FeVCToy";
import { percent } from "../utils";
import NameBadge from "./NameBadge";
import ToyFeatureForm from "./ToyFeatureForm";

export default function ({
  toy,
  refetchToys,
}: {
  toy: FeVCToy;
  refetchToys: () => void;
}) {
  return (
    <div className="toy-container">
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
              <ToyFeatureForm
                toyId={toy.toy_id}
                feature={feature}
                refetchToys={refetchToys}
              />
            </Accordion.Body>
          </Accordion.Item>
        ))}
      </Accordion>
    </div>
  );
}
