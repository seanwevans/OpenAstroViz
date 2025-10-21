import { differenceInMinutes, format, parseISO } from 'date-fns';
import { useMemo } from 'react';

interface TimelineScrubberProps {
  epoch: string;
  start: string;
  end: string;
  value: number; // seconds offset from epoch
  onChange: (value: number) => void;
  onLive: () => void;
}

export function TimelineScrubber({ epoch, start, end, value, onChange, onLive }: TimelineScrubberProps) {
  const range = useMemo(() => {
    const startDate = parseISO(start);
    const endDate = parseISO(end);
    const totalMinutes = differenceInMinutes(endDate, startDate);
    return {
      min: Math.floor((startDate.getTime() - parseISO(epoch).getTime()) / 1000),
      max: Math.ceil((endDate.getTime() - parseISO(epoch).getTime()) / 1000),
      label: `${totalMinutes} min span`
    };
  }, [epoch, start, end]);

  const isLive = Math.abs(value) < 1;

  return (
    <div>
      <div className="timeline-time">
        <div>
          <strong>{format(parseISO(epoch), 'MMM d, HH:mm:ss')}</strong>
          <div>{range.label}</div>
        </div>
        <button className="button" type="button" onClick={onLive} disabled={isLive}>
          {isLive ? 'Live' : 'Jump to Live'}
        </button>
      </div>
      <input
        className="timeline-slider"
        type="range"
        min={range.min}
        max={range.max}
        step={30}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
      <div className="timeline-time">
        <span>Past</span>
        <span>{format(parseISO(epoch), "HH:mm'Z'")}</span>
        <span>Future</span>
      </div>
    </div>
  );
}
