import { describe, expect, it } from 'vitest';
import { mockSnapshot } from './mockData';

describe('mockSnapshot', () => {
  it('has internally consistent summary stats', () => {
    const debris = mockSnapshot.objects.filter((object) => object.kind === 'debris').length;
    const warnings = mockSnapshot.objects.filter(
      (object) => object.health.status === 'warning',
    ).length;
    const critical = mockSnapshot.objects.filter(
      (object) => object.health.status === 'critical',
    ).length;

    expect(mockSnapshot.stats).toBeDefined();
    expect(mockSnapshot.stats?.tracked).toBe(mockSnapshot.objects.length);
    expect(mockSnapshot.stats?.debris).toBe(debris);
    expect(mockSnapshot.stats?.warnings).toBe(warnings);
    expect(mockSnapshot.stats?.critical).toBe(critical);
  });

  it('uses a valid time window around the epoch', () => {
    const start = Date.parse(mockSnapshot.start);
    const epoch = Date.parse(mockSnapshot.epoch);
    const end = Date.parse(mockSnapshot.end);

    expect(Number.isNaN(start)).toBe(false);
    expect(Number.isNaN(epoch)).toBe(false);
    expect(Number.isNaN(end)).toBe(false);
    expect(start).toBeLessThan(epoch);
    expect(epoch).toBeLessThan(end);
  });

  it('references known object IDs in close approaches', () => {
    const ids = new Set(mockSnapshot.objects.map((object) => object.id));

    for (const approach of mockSnapshot.closeApproaches) {
      expect(ids.has(approach.primaryId)).toBe(true);
      expect(ids.has(approach.secondaryId)).toBe(true);
      expect(approach.missDistanceKm).toBeGreaterThan(0);
      expect(approach.relativeVelocityKps).toBeGreaterThan(0);
    }
  });
});
