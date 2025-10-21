import { useEffect, useMemo, useRef, useState } from 'react';
import { addSeconds, differenceInSeconds, parseISO } from 'date-fns';
import useSWR from 'swr';
import { mockSnapshot } from '../utils/mockData';
import { CloseApproach, OrbitalObject, OrbitalSnapshot, OrbitalStreamMessage } from '../types/orbit';

const fallbackHttpUrl = import.meta.env.VITE_DAEMON_HTTP ?? 'http://localhost:8000';
const fallbackWsUrl = import.meta.env.VITE_DAEMON_WS ?? 'ws://localhost:8000/ws/orbits';

const fetcher = (url: string) => fetch(url).then((res) => {
  if (!res.ok) {
    throw new Error(`Request failed: ${res.status}`);
  }
  return res.json();
});

export interface OrbitalStreamState {
  snapshot: OrbitalSnapshot;
  status: 'connecting' | 'live' | 'offline';
  error?: string;
}

export function useOrbitalStream(): OrbitalStreamState {
  const [snapshot, setSnapshot] = useState<OrbitalSnapshot>(mockSnapshot);
  const [status, setStatus] = useState<OrbitalStreamState['status']>('connecting');
  const [error, setError] = useState<string>();
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectRef = useRef<number>();

  useEffect(() => {
    const controller = new AbortController();
    function connect() {
      try {
        const ws = new WebSocket(fallbackWsUrl);
        wsRef.current = ws;
        setStatus('connecting');
        ws.onopen = () => {
          setStatus('live');
          setError(undefined);
          ws.send(JSON.stringify({ kind: 'subscribe', stream: 'orbits' }));
        };
        ws.onmessage = (event) => {
          try {
            const payload: OrbitalStreamMessage = JSON.parse(event.data);
            if (payload.kind === 'snapshot' && payload.snapshot) {
              setSnapshot(applySnapshot(payload.snapshot));
            } else if (payload.kind === 'update' && payload.update) {
              setSnapshot((prev) => applyUpdate(prev, payload.update!));
            }
          } catch (err) {
            console.error('Failed to parse orbital stream message', err);
          }
        };
        ws.onerror = () => {
          setStatus('offline');
        };
        ws.onclose = () => {
          setStatus('offline');
          if (!controller.signal.aborted) {
            reconnectRef.current = window.setTimeout(connect, 5_000);
          }
        };
      } catch (err) {
        console.error('Failed to open orbital websocket', err);
        setStatus('offline');
        setError(err instanceof Error ? err.message : String(err));
      }
    }

    connect();

    return () => {
      controller.abort();
      if (reconnectRef.current) {
        window.clearTimeout(reconnectRef.current);
      }
      wsRef.current?.close();
    };
  }, []);

  // HTTP fallback snapshot loader – keeps state fresh when websocket is offline
  const { data } = useSWR<OrbitalSnapshot>(
    status === 'offline' ? `${fallbackHttpUrl}/api/orbits/snapshot` : null,
    fetcher,
    {
      refreshInterval: 60_000,
      onError(err) {
        setError(err.message);
      },
      shouldRetryOnError: false
    }
  );

  useEffect(() => {
    if (data) {
      setSnapshot(applySnapshot(data));
    }
  }, [data]);

  return useMemo(
    () => ({
      snapshot,
      status: status === 'connecting' && snapshot !== mockSnapshot ? 'live' : status,
      error
    }),
    [snapshot, status, error]
  );
}

function applySnapshot(snapshot: OrbitalSnapshot): OrbitalSnapshot {
  const sortedCloseApproaches = [...snapshot.closeApproaches].sort((a, b) =>
    parseISO(a.time).getTime() - parseISO(b.time).getTime()
  );

  return {
    ...snapshot,
    objects: snapshot.objects.map((object) => ({
      ...object,
      closeApproaches: sortedCloseApproaches.filter(
        (approach) => approach.primaryId === object.id || approach.secondaryId === object.id
      )
    })),
    closeApproaches: sortedCloseApproaches
  };
}

function applyUpdate(
  snapshot: OrbitalSnapshot,
  update: NonNullable<OrbitalStreamMessage['update']>
): OrbitalSnapshot {
  const objectMap = new Map(snapshot.objects.map((object) => [object.id, object] as const));
  update.objects.forEach((objectUpdate) => {
    const existing = objectMap.get(objectUpdate.id);
    if (existing) {
      objectMap.set(objectUpdate.id, {
        ...existing,
        position: objectUpdate.position ?? existing.position,
        velocity: objectUpdate.velocity ?? existing.velocity,
        health: objectUpdate.health ?? existing.health
      });
    }
  });

  const epoch = update.epoch ?? snapshot.epoch;
  const deltaSeconds = differenceInSeconds(parseISO(epoch), parseISO(snapshot.epoch));
  const propagatedObjects = Array.from(objectMap.values()).map((object) => ({
    ...object,
    position: propagate(object, deltaSeconds)
  }));

  const closeApproaches: CloseApproach[] = update.closeApproaches
    ? mergeCloseApproaches(snapshot.closeApproaches, update.closeApproaches)
    : snapshot.closeApproaches;

  const stats = snapshot.stats ?? {
    tracked: propagatedObjects.length,
    debris: propagatedObjects.filter((o) => o.kind === 'debris').length,
    warnings: propagatedObjects.filter((o) => o.health.status === 'warning').length,
    critical: propagatedObjects.filter((o) => o.health.status === 'critical').length
  };

  return {
    ...snapshot,
    epoch,
    end: snapshot.end ? addSeconds(parseISO(snapshot.end), deltaSeconds).toISOString() : snapshot.end,
    start: snapshot.start
      ? addSeconds(parseISO(snapshot.start), deltaSeconds).toISOString()
      : snapshot.start,
    objects: propagatedObjects,
    closeApproaches,
    stats: {
      ...stats,
      tracked: propagatedObjects.length,
      debris: propagatedObjects.filter((o) => o.kind === 'debris').length,
      warnings: propagatedObjects.filter((o) => o.health.status === 'warning').length,
      critical: propagatedObjects.filter((o) => o.health.status === 'critical').length
    }
  };
}

function propagate(object: OrbitalObject, deltaSeconds: number): [number, number, number] {
  const [x, y, z] = object.position;
  const [vx, vy, vz] = object.velocity;
  return [x + vx * deltaSeconds, y + vy * deltaSeconds, z + vz * deltaSeconds];
}

function mergeCloseApproaches(
  existing: CloseApproach[],
  updates: CloseApproach[]
): CloseApproach[] {
  const result = new Map<string, CloseApproach>();
  existing.forEach((approach) => {
    const key = `${approach.primaryId}-${approach.secondaryId}-${approach.time}`;
    result.set(key, approach);
  });
  updates.forEach((approach) => {
    const key = `${approach.primaryId}-${approach.secondaryId}-${approach.time}`;
    result.set(key, approach);
  });
  return Array.from(result.values()).sort(
    (a, b) => parseISO(a.time).getTime() - parseISO(b.time).getTime()
  );
}
