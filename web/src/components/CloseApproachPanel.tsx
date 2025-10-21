import { formatDistanceToNowStrict, parseISO } from 'date-fns';
import { CloseApproach } from '../types/orbit';

interface CloseApproachPanelProps {
  approaches: CloseApproach[];
  onHover?: (pair?: { primaryId: string; secondaryId: string }) => void;
}

export function CloseApproachPanel({ approaches, onHover }: CloseApproachPanelProps) {
  if (!approaches.length) {
    return (
      <div className="panel">
        <h2>Close Approaches</h2>
        <p>No conjunction events detected in this window.</p>
      </div>
    );
  }

  return (
    <div className="panel">
      <h2>Close Approaches</h2>
      <div className="close-approach-list">
        {approaches.map((approach) => {
          const time = parseISO(approach.time);
          return (
            <div
              key={`${approach.primaryId}-${approach.secondaryId}-${approach.time}`}
              className="close-approach-item"
              onMouseEnter={() =>
                onHover?.({ primaryId: approach.primaryId, secondaryId: approach.secondaryId })
              }
              onMouseLeave={() => onHover?.()}
            >
              <div className="close-approach-meta">
                <strong>{approach.primaryId}</strong>
                <span>{formatDistanceToNowStrict(time, { addSuffix: true })}</span>
              </div>
              <div className="close-approach-meta">
                <strong>{approach.secondaryId}</strong>
                <span>{approach.missDistanceKm.toFixed(2)} km</span>
              </div>
              <div className="close-approach-meta">
                <span>Rel. velocity</span>
                <span>{approach.relativeVelocityKps.toFixed(2)} km/s</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
