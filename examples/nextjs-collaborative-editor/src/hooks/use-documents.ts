import { DocumentData } from "@/lib/types";
import { ArrayStorageHookResult, useArrayStorage } from "@ember-link/react";

export function useDocuments() {
  const documents: ArrayStorageHookResult<DocumentData> = useArrayStorage('documents');
  
  return documents;
}