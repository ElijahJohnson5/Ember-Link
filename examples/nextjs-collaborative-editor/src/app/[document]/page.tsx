'use client';

import { BlockNote } from '@/components/dynamic-block-note';
import { useChannel } from '@ember-link/react';
import { getYjsProviderForChannel } from '@ember-link/yjs-provider';

export default function Document() {
  const channel = useChannel();
  const provider = getYjsProviderForChannel(channel);

  return <BlockNote provider={provider} />;
}
