/**
 * nodeFactory.ts
 * 
 * A Utility module for creating and styling Nodes/Edges.
 * This acts as a centralized configuration for the visual appearance of the graph.
 * 
 * Responsibilities:
 * - ID Generation (Uniqueness logic)
 * - Default Styling per Node Type (Colors)
 * - Factory methods for creating ReactFlow Nodes and Edges
 */
import type { Edge, Node } from 'reactflow';
import { MarkerType, Position } from 'reactflow';

export const NodeFactory = {
    // Generates a stable but unique ID. 
    // Logic: Label + cleaned timestamp. 
    createId: (): string => {
        // We use crypto.randomUUID if available, otherwise fallback
        if (typeof crypto !== 'undefined' && crypto.randomUUID) {
            return crypto.randomUUID();
        }
        return `node_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    },

    getNodeStyle: (name: string): React.CSSProperties => {
        if (name.includes("Data")) return { borderColor: 'var(--color-neon-blue)' };
        if (name.includes("Strategy")) return { borderColor: 'var(--color-neon-green)' };
        if (name.includes("Multiplexer")) return { borderColor: '#d400ff' };
        if (name.includes("Gateway")) return { borderColor: 'var(--color-neon-red)' };
        if (name.includes("Execution")) return { borderColor: 'var(--color-neon-yellow)' };
        return {};
    },

    createDraftNode: (type: string, category: string, position: { x: number, y: number }): Node => {
        // ID is now a stable UUID, independent of the Label
        const id = NodeFactory.createId();

        return {
            id,
            type,
            position,
            sourcePosition: Position.Right,
            targetPosition: Position.Left,
            data: {
                label: category,     // Initial label is the category name
                name: category,      // Mutable user-defined name
                category: category,  // Fixed business logic type
                isDraft: true       // Helper flag
            },
            style: {
                borderStyle: 'dashed',
                ...NodeFactory.getNodeStyle(category)
            }
        };
    },

    createEdge: (source: string, target: string, sourceHandle?: string, targetHandle?: string): Edge => ({
        id: `${source}-${target}`,
        source,
        target,
        sourceHandle,
        targetHandle,
        animated: true,
        style: { stroke: 'var(--color-neon-blue)' },
        markerEnd: { type: MarkerType.ArrowClosed, color: 'var(--color-neon-blue)' },
    })
};
