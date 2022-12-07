import * as Slider from "@radix-ui/react-slider";
import "./Slider.css";

type SliderProps = {
  disabled?: boolean;
  min: number;
  max: number;
  step: number;
  value: number[];
  onValueChange: (e: number[]) => void;
};

export default function (props: SliderProps) {
  return (
    <Slider.Root className="SliderRoot" {...props} aria-label="Volume">
      <Slider.Track className="SliderTrack">
        <Slider.Range className="SliderRange" />
      </Slider.Track>
      <Slider.Thumb className="SliderThumb" />
    </Slider.Root>
  );
}
