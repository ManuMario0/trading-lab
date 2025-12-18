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
export default () => {

    return (
        <aside style={{
            width: '200px',
            padding: '10px 0', // Remove horizontal padding for cleaner look
            borderLeft: '1px solid var(--color-neon-blue)',
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
                Panel
            </div>

        </aside>
    )
}


