import { useMyPresence, useOthers } from '@ember-link/react';
import './page.css';
import { useEffect, useRef, useState } from 'react';
import type { User } from '@ember-link/core';

const COLORS = ['#DC2626', '#D97706', '#059669', '#7C3AED', '#DB2777'];

export const Page = () => {
  const others = useOthers();
  const [myPresence, setMyPresence] = useMyPresence();

  useEffect(() => {
    const listener = (event: PointerEvent) => {
      setMyPresence({
        cursor: { x: Math.round(event.clientX), y: Math.round(event.clientY) }
      });
    };

    document.addEventListener('pointermove', listener);

    return () => {
      document.removeEventListener('pointermove', listener);
    };
  }, [setMyPresence]);

  useEffect(() => {
    const listener = () => {
      setMyPresence({ cursor: null });
    };

    document.addEventListener('pointerleave', listener);

    return () => {
      document.removeEventListener('pointerleave', listener);
    };
  });

  return (
    <div className="whole-page">
      <div>
        {myPresence && myPresence.cursor
          ? `${myPresence.cursor.x} × ${myPresence.cursor.x}`
          : 'Move your cursor to broadcast its position to other people in the Channel.'}
      </div>

      <div className="cursors-container">
        {others.map((other) => {
          return <Cursor key={other.clientId} user={other} />;
        })}
      </div>
    </div>
  );
};

/**
 * Each cursor maintains a *rendered* position separate from its
 * *target* position from presence, and lerps the rendered position
 * toward the target every animation frame. That smooths the motion
 * between throttled presence updates the way Figma/Notion do.
 *
 * SMOOTHING = 0.25 catches up within ~4-5 frames (~70ms at 60Hz).
 * Higher = snappier, lower = smoother but laggier.
 */
const SMOOTHING = 0.25;

const Cursor = ({ user }: { user: User }) => {
  const [color] = useState(COLORS[Math.floor(Math.random() * COLORS.length)]);
  const svgRef = useRef<SVGSVGElement | null>(null);

  // Render state lives in a ref so the rAF loop can read/write without
  // triggering React re-renders (React's render path is way too slow
  // for cursor animation).
  const stateRef = useRef({ rx: 0, ry: 0, tx: 0, ty: 0, initialized: false });

  // Update the *target* whenever a new presence arrives. The rAF loop
  // below picks it up.
  useEffect(() => {
    if (!user.cursor) return;
    const state = stateRef.current;
    if (!state.initialized) {
      state.rx = user.cursor.x;
      state.ry = user.cursor.y;
      state.initialized = true;
      if (svgRef.current) {
        svgRef.current.style.transform = `translate3d(${state.rx}px, ${state.ry}px, 0)`;
      }
    }
    state.tx = user.cursor.x;
    state.ty = user.cursor.y;
  }, [user.cursor?.x, user.cursor?.y]);

  // One rAF loop per cursor that lerps until it reaches the target.
  // We keep it running continuously so the cursor stays in motion even
  // if presence updates are delayed.
  useEffect(() => {
    let raf = 0;
    const tick = () => {
      const state = stateRef.current;
      const dx = state.tx - state.rx;
      const dy = state.ty - state.ry;
      if (Math.abs(dx) >= 0.1 || Math.abs(dy) >= 0.1) {
        state.rx += dx * SMOOTHING;
        state.ry += dy * SMOOTHING;
        if (svgRef.current) {
          svgRef.current.style.transform = `translate3d(${state.rx}px, ${state.ry}px, 0)`;
        }
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, []);

  if (!user.cursor) {
    return null;
  }

  return (
    <svg
      ref={svgRef}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke={color}
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      className="cursor lucide lucide-mouse-pointer2-icon lucide-mouse-pointer-2"
    >
      <path d="M4.037 4.688a.495.495 0 0 1 .651-.651l16 6.5a.5.5 0 0 1-.063.947l-6.124 1.58a2 2 0 0 0-1.438 1.435l-1.579 6.126a.5.5 0 0 1-.947.063z" />
    </svg>
  );
};
