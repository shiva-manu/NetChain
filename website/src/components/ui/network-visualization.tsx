import { cn } from "@/lib/utils";

interface NetworkVisualizationProps {
  className?: string;
  nodeCount?: number;
}

export function NetworkVisualization({
  className,
  nodeCount = 12,
}: NetworkVisualizationProps) {
  const nodes = Array.from({ length: nodeCount }, (_, i) => {
    const angle = (i / nodeCount) * Math.PI * 2;
    const radius = 35 + Math.sin(i * 1.7) * 15;
    const x = 50 + Math.cos(angle) * radius;
    const y = 50 + Math.sin(angle) * radius;
    const isPrimary = i < 3;
    const size = isPrimary ? 3 : 1.5;
    return { x, y, size, isPrimary, id: i };
  });

  const connections: { x1: number; y1: number; x2: number; y2: number; key: string }[] = [];
  for (let i = 0; i < nodes.length; i++) {
    const next = (i + 1) % nodes.length;
    connections.push({
      x1: nodes[i].x, y1: nodes[i].y,
      x2: nodes[next].x, y2: nodes[next].y,
      key: `${i}-${next}`,
    });
    if (i < nodes.length / 2) {
      const cross = (i + Math.floor(nodeCount / 3)) % nodes.length;
      connections.push({
        x1: nodes[i].x, y1: nodes[i].y,
        x2: nodes[cross].x, y2: nodes[cross].y,
        key: `${i}-${cross}`,
      });
    }
  }

  return (
    <svg
      viewBox="0 0 100 100"
      className={cn("w-full h-full", className)}
      xmlns="http://www.w3.org/2000/svg"
    >
      {connections.map((conn) => (
        <line
          key={conn.key}
          x1={conn.x1}
          y1={conn.y1}
          x2={conn.x2}
          y2={conn.y2}
          stroke="currentColor"
          className="text-primary/20"
          strokeWidth="0.3"
        />
      ))}
      {nodes.map((node) => (
        <g key={node.id}>
          {node.isPrimary && (
            <circle
              cx={node.x}
              cy={node.y}
              r={node.size * 2.5}
              className="fill-primary/10 animate-pulse-signal"
              style={{ animationDelay: `${node.id * 200}ms` }}
            />
          )}
          <circle
            cx={node.x}
            cy={node.y}
            r={node.size}
            className={node.isPrimary ? "fill-primary" : "fill-primary/40"}
          />
        </g>
      ))}
    </svg>
  );
}
