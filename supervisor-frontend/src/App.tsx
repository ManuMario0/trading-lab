import { useState } from 'react';
import 'reactflow/dist/style.css';
import OrchestratorBoard, { type OrchestratorControls } from './features/orchestrator/OrchestratorBoard';

const headerBtnStyle = {
  background: 'rgba(0, 243, 255, 0.1)',
  border: '1px solid var(--color-neon-blue)',
  color: 'var(--color-neon-blue)',
  padding: '6px 12px',
  borderRadius: '4px',
  cursor: 'pointer',
  fontFamily: 'var(--font-mono)',
  fontSize: '0.8rem',
  textTransform: 'uppercase' as const,
  marginLeft: '10px'
};

const headerBtnPrimaryStyle = {
  ...headerBtnStyle,
  background: 'var(--color-neon-blue)',
  color: 'var(--color-bg-dark)',
  fontWeight: 'bold' as const
};

function App() {
  const [controls, setControls] = useState<OrchestratorControls | null>(null);

  return (
    <div style={{ width: '100vw', height: '100vh', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <div className="header-bar">
        <div style={{ display: 'flex', alignItems: 'center', gap: 20 }}>
          <div className="logo">Antigravity<span style={{ color: 'white' }}>.Compute</span></div>
          <div className="status-indicator">
            <div className="status-dot"></div>
            <span style={{ fontSize: '0.8em', color: '#888' }}>Supervision Console</span>
          </div>
        </div>

        {/* Action Area */}
        {controls && (
          <div style={{ display: 'flex', alignItems: 'center' }}>
            <button style={headerBtnStyle} onClick={controls.onClear}>Clear</button>
            <div style={{ width: 1, height: 20, background: '#333', margin: '0 15px' }}></div>
            <button style={headerBtnPrimaryStyle} onClick={controls.onDeploy}>DEPLOY SYSTEM</button>
          </div>
        )}
      </div>

      <OrchestratorBoard setControls={setControls} />

    </div >
  );
}

export default App;
