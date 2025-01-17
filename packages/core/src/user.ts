import { DefaultPresence } from './index.js';

export type User<P extends Record<string, unknown> = DefaultPresence> = P & {
  clientId: string;
};
