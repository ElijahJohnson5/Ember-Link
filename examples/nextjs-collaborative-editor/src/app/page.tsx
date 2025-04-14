'use client';

import { Button } from '@/components/ui/button';
import { useCreateDocumentAction } from '@/hooks/use-create-document-action';
import { useStatus } from '@ember-link/react';
import { useEffect } from 'react';

export default function Home() {
  const createDocumentAction = useCreateDocumentAction();
  const status = useStatus();

  useEffect(() => {
    console.log(status);
  }, [status]);

  return (
    <div className="flex w-full h-full items-center justify-center flex-col space-y-6">
      <h1>Create or open a document to get started</h1>
      <Button onClick={createDocumentAction}>Create a document</Button>
    </div>
  );
}
