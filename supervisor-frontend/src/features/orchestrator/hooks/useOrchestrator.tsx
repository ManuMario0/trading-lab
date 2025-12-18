/**
 * useOrchestrator.tsx
 * 
 * The main "Controller" hook for the Orchestrator Board.
 * This hook encapsulates the business logic of the visual editor.
 * 
 * Responsibilities:
 * - Managing the ReactFlow Nodes and Edges state
 * - Integrating with the ProcessMonitor to overlay live status on nodes
 * - Providing high-level actions (deploy, save, addNode) to the View
 * - Handling dynamic label generation (injecting StatusDot components)
 */
import { useCallback, useEffect, useMemo } from 'react';
import type {
    Connection,
    Project,
} from 'reactflow';
import {
    addEdge,
    useEdgesState,
    useNodesState
} from 'reactflow';
import { apiClient } from '../../../api/client';
import { NodeFactory } from '../utils/nodeFactory';
import { useProcessMonitor } from './useProcessMonitor';

// Status Dot Component for the label renderer
// We define it here or in a view file, but for the label JSX generation it's needed in scope or imported.
// For cleaner separation, we'll inline the JSX generation logic in a helper function here.

const StatusDot = ({ active }: { active: boolean }) => (
    <div style={{
        width: 8, height: 8, borderRadius: '50%',
        background: active ? 'var(--color-neon-green)' : '#888',
        boxShadow: active ? '0 0 5px var(--color-neon-green)' : 'none'
    }} />
);

export function useOrchestrator(project?: Project) {
    // 1. Core ReactFlow State
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] = useEdgesState([]);

    // 2. Composition: Process Monitor
    const { processes, error, actions: monitorActions } = useProcessMonitor();

    // 3. Initial Load
    useEffect(() => {
        const loadLayout = async () => {
            try {
                const layout = await apiClient.getLayout("default");
                setNodes(layout.nodes);
                setEdges(layout.edges);
            } catch (e) {
                console.log("No default layout found, starting fresh.");
            }
        };
        loadLayout();
    }, [setNodes, setEdges]);

    // 4. Merge Logic: Nodes + Process Status = Display Nodes
    // We memoize this so ReactFlow only re-renders when data changes.
    const displayNodes = useMemo(() => {
        return nodes.map(node => {
            // New Logic: Match by UUID (node.id) strictly
            const proc = processes.find(p => p.id === node.id);
            const isActive = !!proc;

            // In the new model, node.data.name is the Display Name.
            // If it's missing (legacy), fallback to ID or Category.
            const displayName = node.data.name || node.data.label || node.id;
            const displayCategory = node.data.category || "Unknown";

            return {
                ...node,
                data: {
                    ...node.data,
                    // Inject Dynamic Label Component
                    label: (
                        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                            <StatusDot active={isActive} />
                            <div>
                                <div style={{ fontWeight: 'bold' }}>{displayName}</div>
                                <div style={{ fontSize: '0.8em', color: '#888' }}>
                                    {isActive ? `Status: ${proc?.status}` : (node.data.isDraft ? 'Draft' : 'Offline')}
                                </div>
                                <div style={{ fontSize: '0.6em', color: '#555', marginTop: 2 }}>
                                    {displayCategory}
                                </div>
                            </div>
                        </div>
                    )
                }
            };
        });
    }, [nodes, processes]);

    // 5. Actions (The Public API)

    const save = useCallback(async () => {
        try {
            // We save the raw nodes, not the display nodes (which contain JSX in data)
            // ideally we strip the JSX label before saving or backend ignores it.
            // For safety, let's just save.
            await apiClient.saveLayout("default", nodes, edges);
            console.log("Saved.");
        } catch (e) {
            console.error(e);
        }
    }, [nodes, edges]);

    const deploy = useCallback(async () => {
        await save();
        try {
            await apiClient.deployLayout("default");
            monitorActions.refresh();
        } catch (e) {
            console.error(e);
        }
    }, [monitorActions, save]);

    const clear = useCallback(async () => {
        try {
            setNodes([]);
            setEdges([]);
            await apiClient.saveLayout("default", [], []);
            console.log("Cleared.");
        } catch (e) {
            console.error(e);
        }
    }, [setNodes, setEdges, apiClient]);

    const onConnect = useCallback((params: Connection) => {
        setEdges((eds) => addEdge(params, eds));
    }, [setEdges]);

    const addNode = useCallback((type: string, label: string, clientPos: { x: number, y: number }, reactFlowBounds: DOMRect) => {
        if (!project) return;

        const position = project({
            x: clientPos.x - reactFlowBounds.left,
            y: clientPos.y - reactFlowBounds.top,
        });

        const newNode = NodeFactory.createDraftNode(type, label, position);
        setNodes((nds) => nds.concat(newNode));
    }, [project, setNodes]);

    // 6. Memoized Actions
    const actions = useMemo(() => ({
        deploy,
        save,
        addNode,
        clear
    }), [deploy, save, addNode, clear]);

    return {
        // State
        nodes: displayNodes,
        edges,
        error,

        // ReactFlow Handlers
        onNodesChange,
        onEdgesChange,
        onConnect,

        // High Level Actions
        actions
    };
}
