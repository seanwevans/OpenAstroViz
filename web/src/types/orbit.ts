export type ObjectKind = 'satellite' | 'debris' | 'rocket_body';
export type HealthStatus = 'nominal' | 'warning' | 'critical';

export interface ObjectHealth {
  status: HealthStatus;
  battery: number; // 0-1 scale
  thermal: number; // 0-1 scale
  comms: number; // 0-1 scale
  lastContact: string; // ISO timestamp
}

export interface CloseApproach {
  time: string; // ISO timestamp
  primaryId: string;
  secondaryId: string;
  missDistanceKm: number;
  relativeVelocityKps: number;
}

export interface OrbitalObject {
  id: string;
  noradId: number;
  name: string;
  kind: ObjectKind;
  position: [number, number, number]; // km in ECI
  velocity: [number, number, number]; // km/s in ECI
  health: ObjectHealth;
  closeApproaches?: CloseApproach[];
}

export interface OrbitalSnapshot {
  epoch: string;
  start: string;
  end: string;
  objects: OrbitalObject[];
  closeApproaches: CloseApproach[];
  stats?: {
    tracked: number;
    debris: number;
    warnings: number;
    critical: number;
  };
}

export interface OrbitalStreamMessage {
  kind: 'snapshot' | 'update' | 'telemetry';
  snapshot?: OrbitalSnapshot;
  update?: {
    epoch: string;
    objects: Array<Pick<OrbitalObject, 'id' | 'position' | 'velocity' | 'health'>>;
    closeApproaches?: CloseApproach[];
  };
}
