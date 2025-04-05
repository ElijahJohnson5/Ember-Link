import { useMyPresence, useOthers } from '@ember-link/react';
import './page.css';
import { useEffect, useState } from 'react';
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

const Cursor = ({ user }: { user: User }) => {
  const [color] = useState(COLORS[Math.floor(Math.random() * COLORS.length)]);

  if (!user.cursor) {
    return null;
  }

  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke={color}
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
      style={{ transform: `translateX(${user.cursor.x}px) translateY(${user.cursor.y}px);` }}
      className="cursor lucide lucide-mouse-pointer2-icon lucide-mouse-pointer-2"
    >
      <path d="M4.037 4.688a.495.495 0 0 1 .651-.651l16 6.5a.5.5 0 0 1-.063.947l-6.124 1.58a2 2 0 0 0-1.438 1.435l-1.579 6.126a.5.5 0 0 1-.947.063z" />
    </svg>
  );
};
