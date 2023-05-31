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
      <RadixSlider.Track className="relative bg-gray-700 flex-grow rounded-full h-2">
        <RadixSlider.Range className="absolute bg-gray-100 rounded-full h-full data-[disabled]:bg-gray-600 transition-all" />
      </RadixSlider.Track>
    </RadixSlider.Root>
  );
}
