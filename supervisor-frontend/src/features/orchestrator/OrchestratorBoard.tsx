/**
 * OrchestratorBoard.tsx
 * 
 * The main container component for the System Orchestrator UI.
 * This component acts as the "View" layer, orchestrating the interaction between
 * the ReactFlow canvas, the Sidebar, and the Toolbar.
 * 
 * Responsibilities:
 * - Rendering the main layout (Canvas, Sidebar, Toolbar)
 * - Handling Drag & Drop events (View -> Controller)
 * - context management for ReactFlow
 * - Delegating logic to the 'useOrchestrator' hook
 */
import React, { useCallback, useEffect, useRef } from 'react';
import ReactFlow, { Background, ReactFlowProvider, useReactFlow } from 'reactflow';
import 'reactflow/dist/style.css';

import { ServiceNode } from './components/ServiceNode';
import Sidebar from './components/library/Sidebar';
import Panel from './components/panel/Panel';
import { useOrchestrator } from './hooks/useOrchestrator';

const nodeTypes = {
    serviceNode: ServiceNode
};

export interface OrchestratorControls {
    onSave: () => void;
    onDeploy: () => void;
    onRefresh: () => void;
    onClear: () => void;
}

// Inner component needed to access ReactFlow Context (useReactFlow)
const OrchestratorBoardContent = ({ setControls }: { setControls: (controls: OrchestratorControls) => void }) => {
    const reactFlowWrapper = useRef<HTMLDivElement>(null);
    const { project } = useReactFlow();

    // Initialize Logic Hook
    const { nodes, edges, onNodesChange, onEdgesChange, onConnect, actions } = useOrchestrator(project);

    // Expose controls to parent
    useEffect(() => {
        setControls({
            onSave: actions.save,
            onDeploy: actions.deploy,
            onRefresh: () => window.location.reload(),
            onClear: actions.clear
        });
    }, [actions, setControls]);

    // --- Drag & Drop Handlers (Connecting View to Controller) ---
    const onDragOver = useCallback((event: React.DragEvent) => {
        event.preventDefault();
        event.dataTransfer.dropEffect = 'move';
    }, []);

    const onDrop = useCallback(
        (event: React.DragEvent) => {
            event.preventDefault();
            const type = event.dataTransfer.getData('application/reactflow/type');
            const label = event.dataTransfer.getData('application/reactflow/label');
            const offsetData = event.dataTransfer.getData('application/reactflow/offset');
            const offset = offsetData ? JSON.parse(offsetData) : { x: 0, y: 0 };

            if (!type || !reactFlowWrapper.current) return;

            const clientPos = {
                x: event.clientX - offset.x,
                y: event.clientY - offset.y
            };

            const bounds = reactFlowWrapper.current.getBoundingClientRect();

            actions.addNode(type, label, clientPos, bounds);
        },
        [actions]
    );

    return (
        <div style={{ height: 'calc(100vh - 60px)', display: 'flex' }}>
            <Sidebar />

            <div style={{ flex: 1, position: 'relative' }} ref={reactFlowWrapper}>
                <ReactFlow
                    nodes={nodes}
                    edges={edges}
                    onNodesChange={onNodesChange}
                    onEdgesChange={onEdgesChange}
                    onConnect={onConnect}
                    onDragOver={onDragOver}
                    onDrop={onDrop}
                    nodeTypes={nodeTypes}
                    fitView
                    proOptions={{ hideAttribution: true }}
                    style={{ background: 'var(--color-bg-dark)' }}
                >
                    <Background gap={20} color="#1907b8ff" size={2} />
                </ReactFlow>
            </div>

            <Panel />
        </div>
    );
};

// Main Entry that provides Context
export default function OrchestratorBoard({ setControls }: { setControls: (c: OrchestratorControls) => void }) {
    return (
        <ReactFlowProvider>
            <OrchestratorBoardContent setControls={setControls} />
        </ReactFlowProvider>
    );
}
