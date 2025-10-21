import { addMinutes, subMinutes } from 'date-fns';
import { CloseApproach, OrbitalObject, OrbitalSnapshot } from '../types/orbit';

const baseEpoch = new Date();
const start = subMinutes(baseEpoch, 90);
const end = addMinutes(baseEpoch, 90);

const objects: OrbitalObject[] = [
  {
    id: '25544',
    noradId: 25544,
    name: 'ISS (ZARYA)',
    kind: 'satellite',
    position: [4123.0, 5121.5, -5632.4],
    velocity: [-5.124, 3.134, 2.145],
    health: {
      status: 'nominal',
      battery: 0.92,
      thermal: 0.88,
      comms: 0.94,
      lastContact: baseEpoch.toISOString()
    },
    closeApproaches: []
  },
  {
    id: '43013',
    noradId: 43013,
    name: 'STARLINK-1500',
    kind: 'satellite',
    position: [-6521.1, 1123.4, 2450.2],
    velocity: [1.245, 4.231, -3.221],
    health: {
      status: 'warning',
      battery: 0.62,
      thermal: 0.72,
      comms: 0.58,
      lastContact: subMinutes(baseEpoch, 12).toISOString()
    },
    closeApproaches: []
  },
  {
    id: '39227',
    noradId: 39227,
    name: 'FENGYUN 1C DEB',
    kind: 'debris',
    position: [3210.4, -6800.5, 1210.6],
    velocity: [4.891, -1.112, 2.019],
    health: {
      status: 'critical',
      battery: 0.18,
      thermal: 0.32,
      comms: 0.0,
      lastContact: subMinutes(baseEpoch, 1430).toISOString()
    },
    closeApproaches: []
  },
  {
    id: '51012',
    noradId: 51012,
    name: 'COSMOS 2543',
    kind: 'satellite',
    position: [2345.1, 4210.2, 5122.3],
    velocity: [-3.155, 0.612, 5.122],
    health: {
      status: 'nominal',
      battery: 0.88,
      thermal: 0.91,
      comms: 0.87,
      lastContact: subMinutes(baseEpoch, 3).toISOString()
    },
    closeApproaches: []
  }
];

const closeApproaches: CloseApproach[] = [
  {
    primaryId: '25544',
    secondaryId: '43013',
    time: addMinutes(baseEpoch, 24).toISOString(),
    missDistanceKm: 9.4,
    relativeVelocityKps: 12.1
  },
  {
    primaryId: '43013',
    secondaryId: '39227',
    time: subMinutes(baseEpoch, 35).toISOString(),
    missDistanceKm: 6.2,
    relativeVelocityKps: 10.5
  }
];

export const mockSnapshot: OrbitalSnapshot = {
  epoch: baseEpoch.toISOString(),
  start: start.toISOString(),
  end: end.toISOString(),
  objects,
  closeApproaches,
  stats: {
    tracked: objects.length,
    debris: objects.filter((o) => o.kind === 'debris').length,
    warnings: objects.filter((o) => o.health.status === 'warning').length,
    critical: objects.filter((o) => o.health.status === 'critical').length
  }
};
