import type { Edge, Node } from "reactflow";
import { fromSavedLayout, toSavedLayout } from "../features/orchestrator/utils/layoutPersistence";

export interface ProcessInfo {
    id: string; // The UUID Key
    status: string;
}

export interface SpawnRequest {
    name: string;
    cmd: string;
    args: string[];
}

const API_BASE = "http://localhost:3000";

export const apiClient = {
    async getProcesses(): Promise<ProcessInfo[]> {
        const res = await fetch(`${API_BASE}/processes`);
        if (!res.ok) throw new Error("Failed to fetch processes");
        const data = await res.json();
        return data.processes;
    },

    async saveLayout(layoutId: string, nodes: Node[], edges: Edge[]): Promise<void> {
        const res = await fetch(`${API_BASE}/layout/${layoutId}`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(toSavedLayout(nodes, edges)),
        });
        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.msg || "Failed to save layout");
        }
    },

    async removeLayout(layoutId: string): Promise<void> {
        const res = await fetch(`${API_BASE}/layout/${layoutId}`, {
            method: "DELETE",
        });
        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.msg || "Failed to remove layout");
        }
    },

    async getLayout(layoutId: string): Promise<{ nodes: Node[], edges: Edge[] }> {
        const res = await fetch(`${API_BASE}/layout/${layoutId}`);
        if (!res.ok) throw new Error("Failed to fetch layout");
        const response = await res.json();
        return fromSavedLayout(response.layout);
    },

    async deployLayout(layoutId: string): Promise<void> {
        await fetch(`${API_BASE}/layout/${layoutId}/deploy`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
        });
    }
};
