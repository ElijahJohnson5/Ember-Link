import { ObservableV2 } from 'lib0/observable.js';
import { applyUpdate, type Doc, encodeStateAsUpdate, encodeStateVector } from 'yjs';

interface DocumentHandlerProps {
  doc: Doc;
  syncYDoc: (data: Uint8Array, syncStep: string) => void;
  updateYDoc: (data: Uint8Array) => void;
}

export class DocumentHandler extends ObservableV2<{
  sync: (synced: boolean) => void;
  synced: (synced: boolean) => void;
}> {
  private doc: Doc;
  private internalSynced: boolean = false;
  private syncYDoc: (data: Uint8Array, syncStep: string) => void;
  private updateYDoc: (data: Uint8Array) => void;

  constructor({ doc, syncYDoc, updateYDoc }: DocumentHandlerProps) {
    super();
    this.doc = doc;

    this.doc.on('update', this.handleUpdate.bind(this));

    this.syncYDoc = syncYDoc;
    this.updateYDoc = updateYDoc;

    this.syncDoc();
  }

  public syncDoc() {
    this.synced = false;

    const updateVector = encodeStateVector(this.doc);

    this.syncYDoc(updateVector, 'SyncStep1');
  }

  public handleServerUpdate({ update, syncType }: { update: Uint8Array; syncType?: string }) {
    if (!syncType) {
      applyUpdate(this.doc, update, 'backend');
    }

    if (syncType === 'SyncStep1') {
      this.syncYDoc(encodeStateAsUpdate(this.doc, update), 'SyncStep2');
    } else if (syncType === 'SyncStep2') {
      applyUpdate(this.doc, update, 'backend');
    } else if (syncType === 'SyncDone') {
      this.synced = true;
    }
  }

  private handleUpdate(update: Uint8Array, origin: string) {
    if (origin !== 'backend') {
      this.updateYDoc(update);
    }
  }

  get synced() {
    return this.internalSynced;
  }

  set synced(value: boolean) {
    if (this.internalSynced === value) {
      return;
    }

    this.emit('sync', [value]);
    this.emit('synced', [value]);
  }

  destroy(): void {
    this.doc.off('update', this.handleUpdate);
    this.doc.destroy();
    this._observers = new Map();

    super.destroy();
  }
}
