/**
 * OrchestratorToolbar.tsx
 * 
 * A pure presentation component for the top-right action bar.
 * Contains primary actions like Saving, Deploying, and Refreshing the board.
 */
import React from 'react';

interface ToolbarProps {
    onSave: () => void;
    onDeploy: () => void;
    onRefresh: () => void;
    onClear: () => void;
}

export const OrchestratorToolbar: React.FC<ToolbarProps> = ({ onSave, onDeploy, onRefresh, onClear }) => {
    return (
        <div style={{ position: 'absolute', top: 20, right: 230, zIndex: 10, display: 'flex', gap: 10 }}>
            <button onClick={onSave} style={btnStyle}>
                SAVE BLUEPRINT
            </button>
            <button onClick={onRefresh} style={btnSecondaryStyle}>
                REFRESH UI
            </button>
            <button onClick={onDeploy} style={btnSecondaryStyle}>
                DEPLOY
            </button>
            <button onClick={onClear} style={btnSecondaryStyle}>
                CLEAR
            </button>
        </div>
    );
};

const btnStyle: React.CSSProperties = {
    padding: '8px 16px', background: 'var(--color-bg-panel)',
    border: '1px solid var(--color-neon-blue)', color: 'var(--color-neon-blue)',
    cursor: 'pointer', fontFamily: 'var(--font-mono)'
};

const btnSecondaryStyle: React.CSSProperties = {
    padding: '8px 16px', background: 'var(--color-bg-panel)',
    border: '1px solid #777', color: '#ccc',
    cursor: 'pointer', fontFamily: 'var(--font-mono)'
};
