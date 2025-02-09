import type { FeProcessingMode } from "src-tauri/bindings/FeProcessingMode";
import type { PenetrationSystemType } from "src-tauri/bindings/PenetrationSystemType";
import type { FeVCToyAnatomy } from "../../src-tauri/bindings/FeVCToyAnatomy";

/*
This file is a workaround because we cannot create an iterable string array from a ts_rs exported enum type
We need to define the const string array here, so we can import it into the app
We can enforce a compiler error if the infered type from the array and the export from src-tauri/bindings do not match
*/

// Generic checks
type TypesAreEqual<T, U> = [T] extends [U]
  ? [U] extends [T]
    ? true
    : false
  : false;
type StaticAssert<T extends true> = T;

// Const string arrays
export const ToyAnatomyArray = [
  "Anus",
  "Breasts",
  "Buttocks",
  "Chest",
  "Clitoris",
  "Face",
  "Feet",
  "FootL",
  "FootR",
  "HandLeft",
  "HandRight",
  "Hands",
  "Labia",
  "Mouth",
  "NA",
  "Nipples",
  "Penis",
  "Perineum",
  "Testicles",
  "Thighs",
  "Vagina",
  "Vulva",
  "Wrist",
] as const;
// eslint-disable-next-line @typescript-eslint/no-unused-vars
type AnatomyIsSame = StaticAssert<
  TypesAreEqual<FeVCToyAnatomy, (typeof ToyAnatomyArray)[number]>
>;

export const PenetrationSystems = ["NONE", "TPS", "SPS"] as const;
// eslint-disable-next-line @typescript-eslint/no-unused-vars
type PenTypeIsSame = StaticAssert<
  TypesAreEqual<PenetrationSystemType, (typeof PenetrationSystems)[number]>
>;

export const ProcessingModes = ["Raw", "Smooth", "Rate", "Constant"] as const;
// eslint-disable-next-line @typescript-eslint/no-unused-vars
type ProcessingModeIsSame = StaticAssert<
  TypesAreEqual<FeProcessingMode, (typeof ProcessingModes)[number]>
>;
