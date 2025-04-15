'use client';

import { ChannelProvider, EmberLinkProvider } from '@ember-link/react';
import { createYJSStorageProvider } from '@ember-link/yjs-storage';
import { useMemo } from 'react';

export function LayoutProvider({ children }: { children: React.ReactNode }) {
  const provider = useMemo(() => createYJSStorageProvider(), []);

  return (
    <EmberLinkProvider baseUrl="https://ember-link-sandbox.onrender.com">
      <ChannelProvider
        channelName="notion-clone"
        options={{
          storageProvider: provider
        }}
      >
        {children}
      </ChannelProvider>
    </EmberLinkProvider>
  );
}
