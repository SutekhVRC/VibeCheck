import * as RadixSlider from "@radix-ui/react-slider";

type SliderProps = {
  disabled?: boolean;
  min: number;
  max: number;
  step: number;
  value: number[];
  onValueChange: (e: number[]) => void;
};

export default function Slider(props: SliderProps) {
  return (
    <RadixSlider.Root
      className="relative flex items-center"
      {...props}
      aria-label="Slider"
    >
      <RadixSlider.Track className="relative bg-gray-800 flex-grow rounded-full h-1">
        <RadixSlider.Range className="absolute bg-gray-100 rounded-full h-full data-[disabled]:bg-gray-600" />
      </RadixSlider.Track>
      <RadixSlider.Thumb className="block w-3 h-3 bg-gray-100 rounded-xl data-[disabled]:bg-gray-600 focus:[box-shadow:0_0_0_0.25rem_rgba(0,_0,_0,_0.25)]" />
    </RadixSlider.Root>
  );
}
