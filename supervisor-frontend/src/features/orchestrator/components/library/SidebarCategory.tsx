import React, { useState } from 'react';

interface SidebarCategoryProps {
    title: string;
    children: React.ReactNode;
    defaultOpen?: boolean;
    color?: string; // Optional accent color
}

export const SidebarCategory: React.FC<SidebarCategoryProps> = ({ title, children, defaultOpen = false, color = 'var(--color-neon-blue)' }) => {
    const [isOpen, setIsOpen] = useState(defaultOpen);

    return (
        <div style={{ width: '100%', marginBottom: '10px' }}>
            {/* Header */}
            <div
                onClick={() => setIsOpen(!isOpen)}
                style={{
                    display: 'flex',
                    alignItems: 'center',
                    cursor: 'pointer',
                    padding: '8px 10px',
                    background: 'rgba(255, 255, 255, 0.05)',
                    borderLeft: `3px solid ${color}`,
                    marginBottom: isOpen ? '10px' : '0',
                    transition: 'all 0.2s ease'
                }}
            >
                <div style={{
                    marginRight: '8px',
                    fontSize: '0.7em',
                    transform: isOpen ? 'rotate(90deg)' : 'rotate(0deg)',
                    transition: 'transform 0.2s ease',
                    color: '#888'
                }}>
                    ▶
                </div>
                <span style={{
                    fontSize: '0.9em',
                    fontWeight: 'bold',
                    color: '#eee',
                    fontFamily: 'var(--font-mono)',
                    textTransform: 'uppercase'
                }}>
                    {title}
                </span>
            </div>

            {/* Content */}
            {isOpen && (
                <div style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    gap: '10px',
                    paddingBottom: '10px'
                }}>
                    {children}
                </div>
            )}
        </div>
    );
};
