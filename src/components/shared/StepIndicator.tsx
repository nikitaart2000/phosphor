export interface Step {
  label: string;
  status: 'ok' | 'running' | 'pending' | 'error';
}

interface StepIndicatorProps {
  steps: Step[];
}

function getStatusTag(status: Step['status']): { text: string; color: string } {
  switch (status) {
    case 'ok':
      return { text: '[OK]', color: 'var(--green-bright)' };
    case 'running':
      return { text: '[RUNNING]', color: 'var(--amber)' };
    case 'pending':
      return { text: '[PENDING]', color: 'var(--green-dim)' };
    case 'error':
      return { text: '[FAILED]', color: 'var(--red-bright)' };
  }
}

export function StepIndicator({ steps }: StepIndicatorProps) {
  const total = steps.length;

  return (
    <div
      style={{
        fontFamily: 'var(--font-mono)',
        fontSize: '13px',
        lineHeight: '1.8',
      }}
    >
      {steps.map((step, i) => {
        const tag = getStatusTag(step.status);
        const num = `[${i + 1}/${total}]`;
        const labelLen = step.label.length;
        const dotsNeeded = Math.max(2, 24 - labelLen);
        const dots = '.'.repeat(dotsNeeded);

        return (
          <div key={i} style={{ whiteSpace: 'pre' }}>
            <span style={{ color: 'var(--green-mid)' }}>{num} </span>
            <span style={{ color: 'var(--green-mid)' }}>{step.label} </span>
            <span style={{ color: 'var(--green-dim)' }}>{dots} </span>
            <span style={{ color: tag.color }}>{tag.text}</span>
          </div>
        );
      })}
    </div>
  );
}
