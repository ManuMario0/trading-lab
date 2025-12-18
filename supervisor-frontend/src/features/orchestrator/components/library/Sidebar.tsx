/**
 * Sidebar.tsx
 * 
 * The palette of available microservices (Nodes) that can be dragged onto the board.
 * Each item in the sidebar represents a "Draft" of a specific service type.
 * 
 * Responsibilities:
 * - Rendering drag-sources for each node type
 * - Setting dataTransfer properties for Drag & Drop logic
 */
import React from 'react';
import { SidebarCategory } from './SidebarCategory';

// --- Visual Replica of ServiceNode for Sidebar ---
const SidebarNodePreview = ({ label, borderColor = 'var(--color-neon-blue)' }: { label: string, borderColor?: string }) => (
    <div style={{
        background: 'var(--color-bg-panel)',
        border: `1px solid ${borderColor}`,
        borderRadius: '4px',
        width: '140px', // Slightly smaller than real node (180px)
        height: '140px',
        color: 'white',
        boxShadow: `0 0 5px ${borderColor}`, // Defined glow
        fontFamily: 'var(--font-mono)',
        position: 'relative',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        marginBottom: '15px',
        cursor: 'grab',
        fontSize: '0.8rem'
    }}>
        {/* Fake Handles (Visual Only) */}

        {/* Left Side */}
        <div style={{ position: 'absolute', left: -4, top: '45%', width: 8, height: 8, borderRadius: '50%', background: 'var(--color-neon-blue)' }} />
        <div style={{ position: 'absolute', left: 4, top: '43%', fontSize: '0.6em', color: '#888' }}>IN</div>

        <div style={{ position: 'absolute', left: -4, top: '70%', width: 8, height: 8, borderRadius: '50%', background: 'var(--color-neon-red)' }} />
        <div style={{ position: 'absolute', left: 4, top: '68%', fontSize: '0.6em', color: '#888' }}>CMD</div>

        {/* Right Side */}
        <div style={{ position: 'absolute', right: -4, top: '45%', width: 8, height: 8, borderRadius: '50%', background: 'var(--color-neon-green)' }} />
        <div style={{ position: 'absolute', right: 4, top: '43%', fontSize: '0.6em', color: '#888' }}>OUT</div>

        <div style={{ position: 'absolute', right: -4, top: '70%', width: 8, height: 8, borderRadius: '50%', background: '#d400ff' }} />
        <div style={{ position: 'absolute', right: 4, top: '68%', fontSize: '0.6em', color: '#888' }}>LOG</div>

        {/* Label */}
        <div style={{ textAlign: 'center' }}>
            <strong>{label}</strong>
            <div style={{ fontSize: '0.8em', color: '#aaa', marginTop: 4 }}>Blueprint</div>
        </div>
    </div>
);

export default () => {
    const onDragStart = (event: React.DragEvent, nodeType: string, label: string) => {
        const target = event.target as HTMLDivElement;
        const rect = target.getBoundingClientRect();
        const offsetX = event.clientX - rect.left;
        const offsetY = event.clientY - rect.top;
        event.dataTransfer.setData('application/reactflow/type', nodeType);
        event.dataTransfer.setData('application/reactflow/label', label);
        event.dataTransfer.setData('application/reactflow/offset', JSON.stringify({ x: offsetX, y: offsetY }));
        event.dataTransfer.effectAllowed = 'move';
    }

    return (
        <aside style={{
            width: '200px',
            padding: '10px 0', // Remove horizontal padding for cleaner look
            borderRight: '1px solid var(--color-neon-blue)',
            background: '#13131f',
            overflowY: 'auto',
            maxHeight: '100%',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: '5px'
        }}>
            <div style={{
                color: '#666',
                fontSize: '0.7em',
                textTransform: 'uppercase',
                letterSpacing: '1px',
                marginBottom: '10px',
                width: '100%',
                textAlign: 'center',
                borderBottom: '1px solid #333',
                paddingBottom: '10px'
            }}>
                Component Library
            </div>

            <SidebarCategory title="Data Sources" color="var(--color-neon-blue)">
                <div onDragStart={(event) => onDragStart(event, 'serviceNode', 'DataPipeline')} draggable>
                    <SidebarNodePreview label="Data Pipeline" borderColor="var(--color-neon-blue)" />
                </div>
            </SidebarCategory>

            <SidebarCategory title="Strategies" color="var(--color-neon-green)">
                <div onDragStart={(event) => onDragStart(event, 'serviceNode', 'Strategy')} draggable>
                    <SidebarNodePreview label="Strategy" borderColor="var(--color-neon-green)" />
                </div>
            </SidebarCategory>

            <SidebarCategory title="Execution" color="var(--color-neon-yellow)">
                <div onDragStart={(event) => onDragStart(event, 'serviceNode', 'ExecutionEngine')} draggable>
                    <SidebarNodePreview label="Execution" borderColor="var(--color-neon-yellow)" />
                </div>
            </SidebarCategory>

            <SidebarCategory title="Routing" color="#d400ff" defaultOpen={false}>
                <div onDragStart={(event) => onDragStart(event, 'serviceNode', 'Multiplexer')} draggable>
                    <SidebarNodePreview label="Multiplexer" borderColor="#d400ff" />
                </div>
                <div onDragStart={(event) => onDragStart(event, 'serviceNode', 'Gateway')} draggable>
                    <SidebarNodePreview label="Gateway" borderColor="var(--color-neon-red)" />
                </div>
            </SidebarCategory>

        </aside>
    )
}


