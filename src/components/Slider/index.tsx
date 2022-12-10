import * as Slider from "@radix-ui/react-slider";

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
    <Slider.Root
      className="relative flex items-center"
      {...props}
      aria-label="Volume"
    >
      <Slider.Track className="relative bg-gray-800 flex-grow rounded-full h-1">
        <Slider.Range className="absolute bg-gray-50 rounded-full h-full data-[disabled]:bg-gray-600" />
      </Slider.Track>
      <Slider.Thumb className="block w-3 h-3 bg-gray-50 rounded-xl data-[disabled]:bg-gray-600 focus:[box-shadow:0_0_0_5px_rgba(0,_0,_0,_0.25)]" />
    </Slider.Root>
  );
}
