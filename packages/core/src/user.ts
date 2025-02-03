import { DefaultPresence } from './index';

export type User<P extends Record<string, unknown> = DefaultPresence> = P & {
  clientId: string;
};
