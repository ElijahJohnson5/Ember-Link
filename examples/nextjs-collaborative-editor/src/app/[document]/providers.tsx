'use client';

import { ChannelProvider } from '@ember-link/react';
import type React from 'react';

export function Providers({ document, children }: { document: string; children: React.ReactNode }) {
  return <ChannelProvider channelName={`notion-doc-${document}`}>{children}</ChannelProvider>;
}
