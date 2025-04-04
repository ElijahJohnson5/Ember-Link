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
      className="cursor"
      id={`cursor-${user.clientId}`}
      width="24"
      height="36"
      viewBox="0 0 24 36"
      style={{
        transform: `translateX(${user.cursor.x}px) translateY(${user.cursor.y}px)`
      }}
      key={user.clientId}
      fill={color}
      xmlns="http://www.w3.org/2000/svg"
    >
      <path d="M5.65376 12.3673H5.46026L5.31717 12.4976L0.500002 16.8829L0.500002 1.19841L11.7841 12.3673H5.65376Z" />
    </svg>
  );
};
