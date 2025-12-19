import type { Edge, Node } from 'reactflow';

// --- Type Definitions (Must match Rust Backend) ---

export interface SavedNode {
    id: string;
    name: string;
    category: string;
    position: [number, number]; // Tuple (f64, f64)
    status: string;
}

export interface SavedEdge {
    id: string;
    source: string;
    target: string;
}

export interface LayoutData {
    id: string;
    nodes: SavedNode[];
    edges: SavedEdge[];
}

// --- Mapper Functions ---

/**
 * Converts ReactFlow UI state to Clean Backend Format
 */
export const toSavedLayout = (id: string, nodes: Node[], edges: Edge[]): LayoutData => {
    return {
        id,
        nodes: nodes.map(n => ({
            id: n.id,
            name: n.data?.name || n.id, // Fallback to ID if name missing
            category: n.data?.category || 'Manual',
            position: [n.position.x, n.position.y],
            status: n.data?.status || 'inactive' // Ensure your nodes store this in data!
        })),
        edges: edges.map(e => ({
            id: e.id,
            source: e.source,
            target: e.target,
        }))
    };
};

/**
 * Converts Backend Format back to ReactFlow UI State
 * Note: You will need to re-attach UI helpers (labels, colors) after loading
 * because the backend only stores raw data.
 */
export const fromSavedLayout = (data: LayoutData): { nodes: any[], edges: Edge[] } => {
    return {
        nodes: data.nodes.map(n => ({
            id: n.id,
            type: 'serviceNode', // Default to serviceNode since backend doesn't store type anymore
            position: { x: n.position[0], y: n.position[1] },
            data: {
                label: n.name, // Use name for label 
                name: n.name,
                category: n.category,
                status: n.status
            }
        })),
        edges: data.edges.map(e => ({
            id: e.id,
            source: e.source,
            target: e.target,
            type: 'default'
        }))
    };
};
