'use client';

import { useCreateBlockNote } from '@blocknote/react';
import { BlockNoteView } from '@blocknote/shadcn';
import '@blocknote/shadcn/style.css';
import { EmberLinkYjsProvider } from '@ember-link/yjs-provider';

export function BlockNote({ provider }: { provider?: EmberLinkYjsProvider }) {
  const editor = useCreateBlockNote({
    ...(provider && {
      collaboration: {
        provider,
        fragment: provider.getYDoc().getXmlFragment('blocknote'),
        user: { name: 'Your Username', color: '#ff0000' }
      }
    }),
    domAttributes: {
      editor: {
        class: 'h-full'
      }
    }
  });

  return <BlockNoteView editor={editor} className="h-full max-w-4xl mx-auto p-6" theme="light" />;
}
