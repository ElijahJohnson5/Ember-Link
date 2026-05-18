'use client';

import { ChannelProvider, EmberLinkProvider } from '@ember-link/react';
import { createYJSStorageProvider } from '@ember-link/yjs-storage';
import { useMemo } from 'react';

export function LayoutProvider({ children }: { children: React.ReactNode }) {
  const storageProvider = useMemo(() => createYJSStorageProvider(), []);

  return (
    <EmberLinkProvider baseUrl="http://localhost:8787" storageProvider={storageProvider}>
      <ChannelProvider channelName="data-grids">{children}</ChannelProvider>
    </EmberLinkProvider>
  );
}
