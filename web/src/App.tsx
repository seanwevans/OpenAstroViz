import { useEffect, useMemo, useState } from 'react';
import { addSeconds, format, parseISO } from 'date-fns';
import { GlobeCanvas } from './components/GlobeCanvas';
import { TimelineScrubber } from './components/TimelineScrubber';
import { CloseApproachPanel } from './components/CloseApproachPanel';
import { SpaceHealthPanel } from './components/SpaceHealthPanel';
import { ObjectDetails } from './components/ObjectDetails';
import { useOrbitalStream } from './hooks/useOrbitalStream';
import { OrbitalObject } from './types/orbit';

export default function App() {
  const { snapshot, status, error } = useOrbitalStream();
  const [timeOffset, setTimeOffset] = useState(0);
  const [isLiveMode, setIsLiveMode] = useState(true);
  const [selectedObjectId, setSelectedObjectId] = useState<string>();
  const [hoveredPair, setHoveredPair] = useState<[string, string] | null>(null);

  useEffect(() => {
    if (isLiveMode) {
      setTimeOffset(0);
    }
  }, [snapshot.epoch, isLiveMode]);

  const propagatedObjects = useMemo(
    () => propagateObjects(snapshot.objects, timeOffset),
    [snapshot.objects, timeOffset]
  );

  const selectedObject = useMemo(
    () => propagatedObjects.find((object) => object.id === selectedObjectId),
    [propagatedObjects, selectedObjectId]
  );

  const highlightIds = useMemo(() => {
    const ids = new Set<string>();
    if (hoveredPair) {
      ids.add(hoveredPair[0]);
      ids.add(hoveredPair[1]);
    }
    if (selectedObjectId) {
      ids.add(selectedObjectId);
    }
    const now = addSeconds(parseISO(snapshot.epoch), timeOffset);
    snapshot.closeApproaches.forEach((approach) => {
      const approachTime = parseISO(approach.time);
      const deltaMinutes = Math.abs((approachTime.getTime() - now.getTime()) / 60000);
      if (deltaMinutes < 30) {
        ids.add(approach.primaryId);
        ids.add(approach.secondaryId);
      }
    });
    return ids;
  }, [hoveredPair, selectedObjectId, snapshot.closeApproaches, snapshot.epoch, timeOffset]);

  const timelineStart = snapshot.start ?? addSeconds(parseISO(snapshot.epoch), -2 * 3600).toISOString();
  const timelineEnd = snapshot.end ?? addSeconds(parseISO(snapshot.epoch), 6 * 3600).toISOString();
  const currentTime = format(addSeconds(parseISO(snapshot.epoch), timeOffset), 'MMM d, HH:mm:ss');

  const handleSelect = (object: OrbitalObject) => {
    setSelectedObjectId(object.id);
  };

  const handleTimeChange = (value: number) => {
    setTimeOffset(value);
    setIsLiveMode(Math.abs(value) < 1);
  };

  const handleLive = () => {
    setTimeOffset(0);
    setIsLiveMode(true);
  };

  return (
    <div className="app-shell">
      <div className="panel" style={{ gridColumn: '1 / -1' }}>
        <div className={`status-indicator ${status === 'offline' ? 'error' : ''}`}>
          <span className="dot" />
          <span>
            {status === 'live'
              ? 'Live orbital stream'
              : status === 'connecting'
                ? 'Connecting to orbital stream…'
                : 'Offline mode (mock telemetry)'}
          </span>
          <span>• {currentTime}</span>
          {error ? <span style={{ color: '#ff6f7d' }}>• {error}</span> : null}
        </div>
        <TimelineScrubber
          epoch={snapshot.epoch}
          start={timelineStart}
          end={timelineEnd}
          value={timeOffset}
          onChange={handleTimeChange}
          onLive={handleLive}
        />
      </div>

      <div className="globe-wrapper">
        <GlobeCanvas objects={propagatedObjects} onSelect={handleSelect} highlightedIds={highlightIds} />
      </div>

      <div className="right-column">
        <SpaceHealthPanel snapshot={snapshot} />
        <ObjectDetails object={selectedObject} />
        <CloseApproachPanel
          approaches={snapshot.closeApproaches}
          onHover={(pair) => {
            if (pair) {
              setHoveredPair([pair.primaryId, pair.secondaryId]);
            } else {
              setHoveredPair(null);
            }
          }}
        />
      </div>
    </div>
  );
}

function propagateObjects(objects: OrbitalObject[], deltaSeconds: number): OrbitalObject[] {
  if (Math.abs(deltaSeconds) < 1) {
    return objects;
  }
  return objects.map((object) => ({
    ...object,
    position: [
      object.position[0] + object.velocity[0] * deltaSeconds,
      object.position[1] + object.velocity[1] * deltaSeconds,
      object.position[2] + object.velocity[2] * deltaSeconds
    ] as [number, number, number]
  }));
}
