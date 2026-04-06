import type { Transition } from "motion/react";

export const springs = {
  snappy: { type: "spring", bounce: 0.2, visualDuration: 0.4 } satisfies Transition,
  smooth: { type: "spring", bounce: 0, visualDuration: 0.5 } satisfies Transition,
  gentle: { type: "spring", bounce: 0.15, visualDuration: 0.5 } satisfies Transition,
} as const;

export const fadeFast = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
} as const;

export const fadeSlideUp = {
  initial: { opacity: 0, y: 4 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 4 },
} as const;

export const slideFromRight = {
  initial: { x: "100%", opacity: 0 },
  animate: { x: 0, opacity: 1 },
  exit: { x: "100%", opacity: 0 },
} as const;

export const scaleIn = {
  initial: { opacity: 0, scale: 0.95 },
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0.95 },
} as const;
