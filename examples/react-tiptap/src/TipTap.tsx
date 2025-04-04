// src/Tiptap.tsx
import type { EmberLinkYjsProvider } from '@ember-link/yjs-provider';
import Collaboration from '@tiptap/extension-collaboration';
import CollaborationCursor from '@tiptap/extension-collaboration-cursor';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import type { Doc } from 'yjs';
import './editor.css';
import { useEffect } from 'react';

const defaultContent = `
  <p>Hi 👋, this is a collaborative document.</p>
  <p>Feel free to edit and collaborate in real-time!</p>
`;

const Tiptap = ({
  provider,
  doc,
  user
}: {
  provider: EmberLinkYjsProvider;
  doc: Doc;
  user: Record<string, string>;
}) => {
  const editor = useEditor(
    {
      enableContentCheck: true,
      onContentError: ({ disableCollaboration }) => {
        disableCollaboration();
      },
      onCreate: ({ editor: currentEditor }) => {
        if (provider.synced) {
          if (currentEditor.isEmpty) {
            currentEditor.commands.setContent(defaultContent);
          }
        } else {
          provider.on('sync', () => {
            if (currentEditor.isEmpty) {
              currentEditor.commands.setContent(defaultContent);
            }
          });
        }
      },
      extensions: [
        StarterKit.configure({
          history: false
        }),

        Collaboration.extend().configure({
          document: doc
        }),

        CollaborationCursor.extend().configure({
          provider
        })
      ]
    },
    [provider]
  );

  useEffect(() => {
    if (editor && user) {
      editor.chain().focus().updateUser(user).run();
    }
  }, [editor, user]);

  return (
    <>
      <EditorContent editor={editor} className="editor" />
    </>
  );
};

export default Tiptap;
