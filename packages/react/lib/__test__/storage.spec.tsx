import { expect, it, describe } from 'vitest';
import { renderHook, RenderHookResult, act } from '@testing-library/react';
import {
  ArrayStorageHookResult,
  MapStorageHookResult,
  useArrayStorage,
  useMapStorage
} from '~/storage';
import { EmberLinkProvider } from '~/ember-link-provider';
import { ChannelProvider } from '~/channel-provider';
import { mockStorageProvider } from './mock-storage-provider';

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <EmberLinkProvider baseUrl="http://localhost:9000">
    <ChannelProvider
      channelName="test"
      options={{
        storageProvider: mockStorageProvider
      }}
    >
      {children}
    </ChannelProvider>
  </EmberLinkProvider>
);

describe('useStorageArray', () => {
  it('should return an array and update when the array is updated', () => {
    const {
      result
    }: RenderHookResult<ArrayStorageHookResult<string>, string> = renderHook(
      () => useArrayStorage('test'),
      {
        wrapper
      }
    );

    expect(result.current.current).toEqual([]);

    act(() => {
      result.current.push('test');
    });

    expect(result.current.current).toEqual(['test']);

    act(() => {
      result.current.insertAt(0, 'before');
    });

    expect(result.current.current).toEqual(['before', 'test']);
  });

  it('should work if the name of the array is updated', () => {
    const {
      result,
      rerender
    }: RenderHookResult<ArrayStorageHookResult<string>, string> = renderHook(
      (arrayName) => useArrayStorage(arrayName),
      {
        initialProps: 'test',
        wrapper
      }
    );

    expect(result.current.current).toEqual([]);

    act(() => {
      result.current.push('test');
    });

    rerender('test2');

    expect(result.current.current).toEqual([]);

    act(() => {
      result.current.insertAt(0, 'before');
    });

    expect(result.current.current).toEqual(['before']);
  });
});

describe('useStorageMap', () => {
  it('should return a map and update when the map is updated', () => {
    const {
      result
    }: RenderHookResult<MapStorageHookResult<string, string>, string> = renderHook(
      () => useMapStorage('test'),
      {
        wrapper
      }
    );

    expect(result.current.current).toEqual(new Map());

    act(() => {
      result.current.set('test', 'value');
    });

    expect(result.current.current.has('test')).toEqual(true);
    expect(result.current.current.get('test')).toEqual('value');

    act(() => {
      result.current.clear();
    });

    expect(result.current.current).toEqual(new Map());
  });

  it('should work if the name of the map is updated', () => {
    const {
      result,
      rerender
    }: RenderHookResult<MapStorageHookResult<string, string>, string> = renderHook(
      (name) => useMapStorage(name),
      {
        initialProps: 'test',
        wrapper
      }
    );

    expect(result.current.current).toEqual(new Map());

    act(() => {
      result.current.set('test', 'value');
    });

    rerender('test2');

    expect(result.current.current).toEqual(new Map());

    act(() => {
      result.current.set('test', 'before');
    });

    expect(result.current.current.get('test')).toEqual('before');
  });
});
