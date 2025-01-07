import { EventEmitter, Listener } from "tseep";

export type Observable<T extends { [event in string | symbol]: Listener }> = {
  subscribe(event: keyof T, callback: T[keyof T]): () => void;

  subscribeOnce(event: keyof T, callback: T[keyof T]): () => void;

  waitUntil(event: keyof T): Promise<Parameters<T[keyof T]>>;
};

export type Emitter<T extends { [event in string | symbol]: Listener }> =
  Observable<T> & {
    emit(event: keyof T, ...args: Parameters<T[keyof T]>): boolean;

    count(): number;
    observable: Observable<T>;
  };

export const createEventEmitter = <
  T extends { [event in string | symbol]: Listener },
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
      let callback: T[keyof T] = <any>(
        function (...args: Parameters<T[keyof T]>) {
          res(args);
        }
      );

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
      waitUntil,
    },
  };
};

export const createBufferedEventEmitter = <
  T extends { [event in string | symbol]: Listener },
>() => {
  const eventEmitter = createEventEmitter<T>();

  let buffer: Record<keyof T, Parameters<T[keyof T]>[]> = {} as Record<
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
  }

  return {
    ...eventEmitter,
    emit: emitOrBuffer,
    pause,
    resume,
  };
};
