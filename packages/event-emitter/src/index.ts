/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-function-type */

import { EventEmitter, Listener } from 'tseep';

export type Observable<T extends { [event in string | symbol]: Listener }> = {
  subscribe<Key extends keyof T>(event: Key, callback: T[Key]): () => void;

  subscribeOnce<Key extends keyof T>(event: Key, callback: T[Key]): () => void;

  waitUntil<Key extends keyof T>(event: Key): Promise<Parameters<T[Key]>>;
};

export type Emitter<T extends { [event in string | symbol]: Listener }> = Observable<T> & {
  emit(event: keyof T, ...args: Parameters<T[keyof T]>): boolean;

  count(): number;
  observable: Observable<T>;
};

export type BufferedEmitter<T extends { [event in string | symbol]: Listener }> = Emitter<T> & {
  pause(event: keyof T): void;
  resume(event: keyof T): void;
};

export const createEventEmitter = <
  T extends { [event in string | symbol]: Listener }
>(): Emitter<T> => {
  const events = new EventEmitter<T>();

  function subscribe(event: keyof T, callback: T[keyof T]) {
    events.on(event, callback);
    return () => {
      events.removeListener(event, callback);
    };
  }

  function subscribeOnce(event: keyof T, callback: T[keyof T]) {
    function wrap<T extends Function>(fn: T): T {
      return <any>function (...args) {
        unsub();
        return fn(...args);
      };
    }

    const unsub = subscribe(event, wrap(callback));

    return unsub;
  }

  async function waitUntil(event: keyof T) {
    let unsub: () => void | undefined;

    return new Promise<Parameters<T[keyof T]>>((res) => {
      const callback: T[keyof T] = <any>function (...args: Parameters<T[keyof T]>) {
        res(args);
      };

      unsub = subscribe(event, callback);
    }).finally(() => unsub?.());
  }

  return {
    emit: events.emit.bind(events),
    subscribe,
    subscribeOnce,
    waitUntil,
    count: events.listenerCount.bind(events),

    observable: {
      subscribe,
      subscribeOnce,
      waitUntil
    }
  };
};

export const createBufferedEventEmitter = <
  T extends { [event in string | symbol]: Listener }
>(): BufferedEmitter<T> => {
  const eventEmitter = createEventEmitter<T>();

  const buffer: Record<keyof T, Parameters<T[keyof T]>[]> = {} as Record<
    keyof T,
    Parameters<T[keyof T]>[] | null
  >;

  function pause(event: keyof T) {
    buffer[event] = [];
  }

  function resume(eventName: keyof T) {
    if (!buffer[eventName]) {
      return;
    }

    for (const event of buffer[eventName]) {
      eventEmitter.emit(eventName, ...event);
    }

    buffer[eventName] = null;
  }

  function emitOrBuffer(event: keyof T, ...args: Parameters<T[keyof T]>) {
    if (buffer[event]) {
      buffer[event].push(args);
    } else {
      eventEmitter.emit(event, ...args);
    }

    return true;
  }

  return {
    ...eventEmitter,
    emit: emitOrBuffer,
    pause,
    resume
  };
};
