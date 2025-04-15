import { v4 as uuid } from 'uuid';
import { useCallback } from 'react';
import { useRouter } from 'next/navigation';
import { useDocuments } from './use-documents';

export function useCreateDocumentAction() {
  const docs = useDocuments();
  const router = useRouter();

  const createDocumentAction = useCallback(() => {
    const newDoc = {
      id: uuid()
    };

    docs.push(newDoc);
    router.push(`/${newDoc.id}`);
  }, [docs, router]);

  return createDocumentAction;
}
