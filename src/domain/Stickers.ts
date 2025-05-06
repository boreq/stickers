import json from '@/../public/stickers/stickers.json';

export interface Sticker {
  filename: string;
  text: string;
}

export enum Shape {
  Rectangular,
  Oval,
  Irregular,
}

export enum Category {
  Cyber,
  Event,
  Project,
  Gender,
}

export const stickers: Sticker[] = json;
