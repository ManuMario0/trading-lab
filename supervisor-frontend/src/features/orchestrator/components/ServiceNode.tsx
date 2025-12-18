/**
 * ServiceNode.tsx
 * 
 * A Custom ReactFlow Node component representing a Microservice in the architecture.
 * 
 * Features:
 * - Multiple Handles for different IO types (Data, Command, Logs)
 * - Visual styling to match the system's "Neon" aesthetic
 * - Displays the label passed via data
 */
import { memo } from 'react';
import { Handle, Position, type NodeProps } from 'reactflow';

export const ServiceNode = memo(({ data }: NodeProps) => {
    return (
        <div style={{
            background: 'var(--color-bg-panel)',
            border: '1px solid var(--color-neon-blue)',
            borderRadius: '4px',
            padding: '10px',
            minWidth: '180px',
            minHeight: '180px',
            color: 'white',
            boxShadow: '0 0 10px rgba(0, 243, 255, 0.2)',
            fontFamily: 'var(--font-mono)'
        }}>
            {/* --- Inputs (Left) --- */}

            {/* 1. Main Input (Top Left) */}
            <Handle
                type="target"
                position={Position.Left}
                id="in-main"
                style={{ top: '45%', background: 'var(--color-neon-blue)' }}
            />
            <div style={{ position: 'absolute', left: 10, top: '43%', fontSize: '0.6em', color: '#888' }}>
                IN
            </div>

            {/* 2. Admin/Control Input (Bottom Left) */}
            <Handle
                type="target"
                position={Position.Left}
                id="in-admin"
                style={{ top: '70%', background: 'var(--color-neon-red)' }}
            />
            <div style={{ position: 'absolute', left: 10, top: '68%', fontSize: '0.6em', color: '#888' }}>
                CMD
            </div>


            {/* --- Content --- */}
            <div style={{ textAlign: 'center', padding: '10px 0' }}>
                {data.label}
            </div>


            {/* --- Outputs (Right) --- */}

            {/* 1. Main Output (Top Right) */}
            <Handle
                type="source"
                position={Position.Right}
                id="out-main"
                style={{ top: '45%', background: 'var(--color-neon-green)' }}
            />
            <div style={{ position: 'absolute', right: 10, top: '43%', fontSize: '0.6em', color: '#888' }}>
                OUT
            </div>

            {/* 2. Secondary Output (Bottom Right) */}
            <Handle
                type="source"
                position={Position.Right}
                id="out-aux"
                style={{ top: '70%', background: '#d400ff' }}
            />
            <div style={{ position: 'absolute', right: 10, top: '68%', fontSize: '0.6em', color: '#888' }}>
                LOG
            </div>
        </div>
    );
});
