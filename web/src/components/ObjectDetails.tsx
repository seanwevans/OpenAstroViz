import { formatDistanceToNowStrict, parseISO } from 'date-fns';
import { OrbitalObject } from '../types/orbit';

interface ObjectDetailsProps {
  object?: OrbitalObject;
}

export function ObjectDetails({ object }: ObjectDetailsProps) {
  if (!object) {
    return (
      <div className="panel">
        <h2>Object Details</h2>
        <p>Select an object on the globe to inspect telemetry.</p>
      </div>
    );
  }

  const altitude = Math.sqrt(
    object.position[0] ** 2 + object.position[1] ** 2 + object.position[2] ** 2
  ) - 6378.137;

  return (
    <div className="panel object-details">
      <div className={`badge ${object.health.status}`}>
        {object.health.status.toUpperCase()}
      </div>
      <h3>{object.name}</h3>
      <dl>
        <dt>NORAD ID</dt>
        <dd>{object.noradId}</dd>
        <dt>Type</dt>
        <dd>{object.kind.replace('_', ' ')}</dd>
        <dt>Altitude</dt>
        <dd>{altitude.toFixed(0)} km</dd>
        <dt>Velocity</dt>
        <dd>
          {Math.sqrt(
            object.velocity[0] ** 2 + object.velocity[1] ** 2 + object.velocity[2] ** 2
          ).toFixed(2)}{' '}
          km/s
        </dd>
        <dt>Battery</dt>
        <dd>{Math.round(object.health.battery * 100)}%</dd>
        <dt>Thermal</dt>
        <dd>{Math.round(object.health.thermal * 100)}%</dd>
        <dt>Comms</dt>
        <dd>{Math.round(object.health.comms * 100)}%</dd>
        <dt>Last Contact</dt>
        <dd>{formatDistanceToNowStrict(parseISO(object.health.lastContact), { addSuffix: true })}</dd>
      </dl>
    </div>
  );
}
