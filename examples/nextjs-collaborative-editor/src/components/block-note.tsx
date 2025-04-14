'use client';

import { useCreateBlockNote } from '@blocknote/react';
import { BlockNoteView } from '@blocknote/shadcn';
import '@blocknote/shadcn/style.css';

export function BlockNote() {
  const editor = useCreateBlockNote({
    domAttributes: {
      editor: {
        class: 'h-full'
      }
    }
  });

  return <BlockNoteView editor={editor} className="h-full" theme="light" />;
}
