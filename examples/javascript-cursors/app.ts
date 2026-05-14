import { createClient, User } from '@ember-link/core';

declare global {
  interface EmberLink {
    Presence: {
      cursor: {
        x: number;
        y: number;
      } | null;
    };
  }
}

const client = createClient({
  baseUrl: 'http://localhost:8787'
});

const { channel } = client.joinChannel('test', {
  // Coalesce pointermove-driven presence updates to ~30Hz so we don't
  // spam the WebSocket with every browser-fired move event. The rAF
  // lerp loop below smooths the rendering between updates.
  presenceThrottle: 33
});

const cursorsContainer = document.getElementById('cursors-container')!;
const text = document.getElementById('text')!;

channel.events.subscribe('presence', (presence) => {
  const cursor: { x: number; y: number } | null =
    (presence?.cursor as { x: number; y: number }) ?? null;

  text.innerHTML = cursor
    ? `${cursor.x} × ${cursor.y}`
    : 'Move your cursor to broadcast its position to other people in the channel.';
});

/**
 * Subscribe to every others presence updates.
 * The callback will be called if you or someone else enters or leaves the channel
 * or when someone presence is updated
 */
channel.events.others.subscribe('join', (user) => {
  console.log('User join: ', user);
  setCursorTarget(user);
});

channel.events.others.subscribe('update', (user) => {
  console.log('User update: ', user);
  setCursorTarget(user);
});

channel.events.others.subscribe('leave', (user) => {
  console.log('User leave: ', user);
  deleteCursor(user);
});

channel.events.others.subscribe('reset', () => {
  cursorsContainer.innerHTML = '';
  cursorStates.clear();
  for (const cursor of document.querySelectorAll('[id^="cursor-]')) {
    if (cursor) {
      cursor.parentNode!.removeChild(cursor);
    }
  }
});

document.addEventListener('pointermove', (event) => {
  channel.updatePresence({
    cursor: { x: Math.round(event.clientX), y: Math.round(event.clientY) }
  });
});

document.addEventListener('pointerleave', () => {
  channel.updatePresence({ cursor: null });
});

const COLORS = ['#DC2626', '#D97706', '#059669', '#7C3AED', '#DB2777'];

/**
 * Each remote cursor has a *rendered* position (rx, ry) and a *target*
 * position (tx, ty). The target is updated whenever a new presence event
 * arrives; the rendered position chases the target on every animation
 * frame by easing a fraction of the remaining distance. That gives
 * Figma-style smooth motion regardless of how irregular the network
 * updates are.
 *
 * SMOOTHING controls how snappy/lazy the cursor is. 0.2 catches up
 * within ~5 frames (~80ms at 60Hz). Higher = snappier. Lower = smoother
 * but more visible lag.
 */
const SMOOTHING = 0.25;

type CursorState = {
  el: HTMLElement;
  rx: number;
  ry: number;
  tx: number;
  ty: number;
  visible: boolean;
};

const cursorStates = new Map<string, CursorState>();

function setCursorTarget(user: User) {
  const cursor = getCursorOrCreate(user.clientId);

  if (user.cursor) {
    let state = cursorStates.get(user.clientId);
    if (!state) {
      // First sighting: jump straight to the position so the cursor
      // doesn't appear to fly in from (0, 0).
      state = {
        el: cursor,
        rx: user.cursor.x,
        ry: user.cursor.y,
        tx: user.cursor.x,
        ty: user.cursor.y,
        visible: true
      };
      cursorStates.set(user.clientId, state);
      applyTransform(state);
    } else {
      state.tx = user.cursor.x;
      state.ty = user.cursor.y;
    }
    if (!state.visible) {
      state.visible = true;
      cursor.style.opacity = '1';
    }
  } else {
    const state = cursorStates.get(user.clientId);
    if (state && state.visible) {
      state.visible = false;
      cursor.style.opacity = '0';
    }
  }
}

function applyTransform(state: CursorState) {
  // Use translate3d to keep the cursor on its own compositor layer.
  state.el.style.transform = `translate3d(${state.rx}px, ${state.ry}px, 0)`;
}

function tick() {
  cursorStates.forEach((state) => {
    // Lerp rendered toward target. When close enough (sub-pixel), snap
    // to avoid endless tiny updates that prevent the GPU layer from
    // settling.
    const dx = state.tx - state.rx;
    const dy = state.ty - state.ry;
    if (Math.abs(dx) < 0.1 && Math.abs(dy) < 0.1) {
      if (state.rx !== state.tx || state.ry !== state.ty) {
        state.rx = state.tx;
        state.ry = state.ty;
        applyTransform(state);
      }
      return;
    }
    state.rx += dx * SMOOTHING;
    state.ry += dy * SMOOTHING;
    applyTransform(state);
  });
  requestAnimationFrame(tick);
}
requestAnimationFrame(tick);

function getCursorOrCreate(connectionId: string): HTMLElement {
  let cursor: HTMLElement | null = document.getElementById(`cursor-${connectionId}`);

  if (cursor == null) {
    cursor = document.getElementById('cursor-template')!.cloneNode(true) as HTMLElement;
    cursor.id = `cursor-${connectionId}`;
    cursor.style.stroke = COLORS[Math.floor(Math.random() * COLORS.length)];
    cursor.style.display = 'block';
    cursorsContainer.appendChild(cursor);
  }

  return cursor;
}

function deleteCursor(user: User) {
  const cursor = document.getElementById(`cursor-${user.clientId}`);
  if (cursor) {
    cursor.parentNode!.removeChild(cursor);
  }
  cursorStates.delete(user.clientId);
}
