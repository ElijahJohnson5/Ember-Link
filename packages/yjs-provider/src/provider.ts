import type { Channel } from '@ember-link/core';
import { ObservableV2 } from 'lib0/observable';
import { type Doc } from 'yjs';
import { Awareness } from '~/awareness';
import { DocumentHandler } from '~/handler';

interface EmberLinkYjsProviderProps {
  channel: Channel;
  doc: Doc;
}

export class EmberLinkYjsProvider extends ObservableV2<{
  sync: (synced: boolean) => void;
  synced: (synced: boolean) => void;
}> {
  private readonly doc: Doc;
  private docHandler: DocumentHandler;
  private readonly channel: Channel;
  public readonly awareness: Awareness;

  private readonly unsubscribers: Array<() => void> = [];

  constructor({ channel, doc }: EmberLinkYjsProviderProps) {
    super();

    this.doc = doc;
    this.channel = channel;

    this.awareness = new Awareness(doc, channel);
    this.docHandler = new DocumentHandler({
      doc,
      syncYDoc: (data: Uint8Array, syncType: string) => {
        channel.syncYDoc({
          update: data.buffer as ArrayBuffer,
          syncType: syncType
        });
      },
      updateYDoc: (data: Uint8Array) => {
        channel.updateYDoc({
          update: data.buffer as ArrayBuffer
        });
      }
    });

    this.unsubscribers.push(
      this.channel.events.subscribe('status', (status) => {
        if (status === 'connected') {
          this.docHandler.syncDoc();
        } else {
          this.docHandler.synced = false;
        }
      })
    );

    this.unsubscribers.push(
      this.channel.events.yjsProvider.subscribe('syncMessage', (message) => {
        this.docHandler.handleServerUpdate({
          update: new Uint8Array(message.update),
          syncType: message.syncType
        });
      })
    );

    this.unsubscribers.push(
      this.channel.events.yjsProvider.subscribe('updateMessage', (message) => {
        this.docHandler.handleServerUpdate({
          update: new Uint8Array(message.update)
        });
      })
    );

    this.docHandler.on('synced', (state) => {
      this.emit('synced', [state]);
      this.emit('sync', [state]);
    });
  }

  get synced(): boolean {
    return this.docHandler.synced;
  }

  destroy(): void {
    this.unsubscribers.forEach((unsub) => unsub());
    this.awareness.destroy();
    this.docHandler.destroy();
    this._observers = new Map();
    super.destroy();
  }

  getYDoc(): Doc {
    return this.doc;
  }

  // Some provider implementations expect to be able to call connect/disconnect, implement as noop
  disconnect(): void {
    // This is a noop
  }

  connect(): void {
    // This is a noop
  }
}
