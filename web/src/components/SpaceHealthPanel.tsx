import { OrbitalSnapshot } from '../types/orbit';

interface SpaceHealthPanelProps {
  snapshot: OrbitalSnapshot;
}

export function SpaceHealthPanel({ snapshot }: SpaceHealthPanelProps) {
  const { stats } = snapshot;
  const tracked = stats?.tracked ?? snapshot.objects.length;
  const warnings = stats?.warnings ?? snapshot.objects.filter((o) => o.health.status === 'warning').length;
  const critical = stats?.critical ?? snapshot.objects.filter((o) => o.health.status === 'critical').length;
  const debris = stats?.debris ?? snapshot.objects.filter((o) => o.kind === 'debris').length;
  const nominal = tracked - warnings - critical;

  return (
    <div className="panel">
      <h2>Space Health</h2>
      <div className="metrics-grid">
        <div className="metric-card">
          <span>Total tracked</span>
          <strong>{tracked.toLocaleString()}</strong>
        </div>
        <div className="metric-card">
          <span>Nominal</span>
          <strong>{nominal.toLocaleString()}</strong>
        </div>
        <div className="metric-card">
          <span>Warnings</span>
          <strong>{warnings.toLocaleString()}</strong>
        </div>
        <div className="metric-card">
          <span>Critical</span>
          <strong>{critical.toLocaleString()}</strong>
        </div>
        <div className="metric-card">
          <span>Tracked debris</span>
          <strong>{debris.toLocaleString()}</strong>
        </div>
      </div>
    </div>
  );
}
