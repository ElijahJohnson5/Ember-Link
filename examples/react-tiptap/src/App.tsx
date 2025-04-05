import './App.css';
import { ChannelProvider, EmberLinkProvider } from '@ember-link/react';
import { Page } from './Page';

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

function App() {
  return (
    <EmberLinkProvider baseUrl="http://localhost:9000">
      <ChannelProvider channelName="test" options={{}}>
        <Page />
      </ChannelProvider>
    </EmberLinkProvider>
  );
}

export default App;
