import { useState, useEffect, useMemo, useCallback, useRef } from "react";
import ForceGraph2D from "react-force-graph-2d";
import type { ForceGraphMethods, NodeObject, LinkObject } from "react-force-graph-2d";
import {
  Swords,
} from "lucide-react";
import { outlineApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { Character, CharacterRelation, CharacterFaction, OutlineNode } from "@/lib/types";

interface GraphNode extends NodeObject {
  id: string;
  name: string;
  race: string;
  faction: string;
  factionRole: string;
  color: string;
  val: number;
}

interface GraphLink extends LinkObject {
  source: string;
  target: string;
  relationshipType: string;
  description: string;
  id: string;
}

const FACTION_COLORS = [
  "#7C3AED",
  "#0891B2",
  "#16A34A",
  "#DC2626",
  "#F59E0B",
  "#EC4899",
  "#8B5CF6",
  "#14B8A6",
  "#F97316",
  "#6366F1",
];

function getFactionColor(faction: string, factionColorMap: Map<string, string>): string {
  if (!faction) return "#475569";
  if (factionColorMap.has(faction)) return factionColorMap.get(faction)!;
  const idx = factionColorMap.size % FACTION_COLORS.length;
  const color = FACTION_COLORS[idx];
  factionColorMap.set(faction, color);
  return color;
}

function chapterInRange(
  chapterId: string | null,
  startId: string | null,
  endId: string | null,
  chapterOrderMap: Map<string, number>,
): boolean {
  if (!chapterId) return true;
  const order = chapterOrderMap.get(chapterId);
  if (order === undefined) return true;
  if (startId) {
    const startOrder = chapterOrderMap.get(startId);
    if (startOrder !== undefined && order < startOrder) return false;
  }
  if (endId) {
    const endOrder = chapterOrderMap.get(endId);
    if (endOrder !== undefined && order > endOrder) return false;
  }
  return true;
}

const CharacterGraph = ({
  projectId,
  characters,
  relations,
  factions,
}: {
  projectId: string;
  characters: Character[];
  relations: CharacterRelation[];
  factions: CharacterFaction[];
}) => {
  const [chapters, setChapters] = useState<OutlineNode[]>([]);
  const [sliderValue, setSliderValue] = useState<number>(100);
  const [maxOrder, setMaxOrder] = useState(1);
  const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null);
  const [hoveredLink, setHoveredLink] = useState<GraphLink | null>(null);
  const fgRef = useRef<ForceGraphMethods<GraphNode, GraphLink>>(null!);

  useEffect(() => {
    outlineApi.list(projectId).then((nodes) => {
      const chapters = nodes.filter((n) => n.node_type === "chapter");
      setChapters(chapters);
      const max = chapters.reduce((m, n) => Math.max(m, n.sort_order), 0);
      setMaxOrder(Math.max(max, 1));
    }).catch(() => {});
  }, [projectId]);

  const chapterOrderMap = useMemo(() => {
    const map = new Map<string, number>();
    chapters.forEach((ch) => map.set(ch.id, ch.sort_order));
    return map;
  }, [chapters]);

  const sliderOrder = Math.round((sliderValue / 100) * maxOrder);

  const factionColorMap = useMemo(() => new Map<string, string>(), []);

  const charFactionMap = useMemo(() => {
    const map = new Map<string, { faction: string; role: string }>();
    factions.forEach((f) => {
      if (chapterInRange(f.start_chapter_id, f.end_chapter_id, null, chapterOrderMap)) {
        map.set(f.character_id, { faction: f.faction_name, role: f.role });
      }
    });
    return map;
  }, [factions, chapterOrderMap]);

  const charRelMap = useMemo(() => {
    const map = new Map<string, CharacterRelation>();
    relations.forEach((r) => {
      if (chapterInRange(null, r.start_chapter_id, r.end_chapter_id, chapterOrderMap)) {
        map.set(r.id, r);
      }
    });
    return map;
  }, [relations, chapterOrderMap]);

  const { nodes, links } = useMemo(() => {
    const connectionCount = new Map<string, number>();
    const relLinks: GraphLink[] = [];

    charRelMap.forEach((rel) => {
      if (rel.char_a_id && rel.char_b_id && rel.char_a_id !== rel.char_b_id) {
        connectionCount.set(rel.char_a_id, (connectionCount.get(rel.char_a_id) || 0) + 1);
        connectionCount.set(rel.char_b_id, (connectionCount.get(rel.char_b_id) || 0) + 1);
        relLinks.push({
          source: rel.char_a_id,
          target: rel.char_b_id,
          relationshipType: rel.relationship_type,
          description: rel.description,
          id: rel.id,
        });
      }
    });

    const graphNodes: GraphNode[] = characters.map((c) => {
      const cf = charFactionMap.get(c.id) || { faction: "", role: "" };
      const color = getFactionColor(cf.faction, factionColorMap);
      const connections = connectionCount.get(c.id) || 0;
      return {
        id: c.id,
        name: c.name,
        race: c.race,
        faction: cf.faction,
        factionRole: cf.role,
        color,
        val: Math.max(6, connections),
      };
    });

    return { nodes: graphNodes, links: relLinks };
  }, [characters, charRelMap, charFactionMap, factionColorMap]);

  const handleNodeClick = useCallback((node: GraphNode) => {
    fgRef.current?.centerAt(node.x ?? 0, node.y ?? 0, 800);
    fgRef.current?.zoom(2, 400);
  }, []);

  const uniqueFactions = useMemo(() => {
    const set = new Set<string>();
    nodes.forEach((n) => { if (n.faction) set.add(n.faction); });
    return Array.from(set);
  }, [nodes]);

  const currentChapterName = useMemo(() => {
    const ch = chapters.find((c) => c.sort_order <= sliderOrder)
      ?.sort_order;
    const found = chapters.filter((c) => c.sort_order === ch).pop();
    return found ? found.title : "";
  }, [chapters, sliderOrder, sliderValue]);

  return (
    <div className="flex h-full flex-col gap-3">
      <div className="flex items-center gap-3 px-1 shrink-0">
        <div className="flex-1 flex flex-col gap-1">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground/60">
              时间线：{currentChapterName}
            </span>
            <span className="text-[10px] text-muted-foreground/40">
              {sliderValue >= 100 ? "显示全部" : `第 ${sliderOrder} 章及之前`}
            </span>
          </div>
          <input
            type="range"
            min={0}
            max={100}
            value={sliderValue}
            onChange={(e) => setSliderValue(Number(e.target.value))}
            className="w-full h-1 accent-primary cursor-pointer"
          />
        </div>
      </div>

      {uniqueFactions.length > 0 && (
        <div className="flex items-center gap-2 px-1 shrink-0 flex-wrap">
          <span className="text-[10px] text-muted-foreground/50 uppercase tracking-wider">势力</span>
          {uniqueFactions.map((f) => (
            <div key={f} className="flex items-center gap-1">
              <div
                className="size-2 rounded-full shrink-0"
                style={{ backgroundColor: getFactionColor(f, factionColorMap) }}
              />
              <span className="text-[11px] text-muted-foreground/70">{f}</span>
            </div>
          ))}
        </div>
      )}

      <div className="flex-1 relative min-h-0 rounded-lg border border-border bg-muted/20">
        <ForceGraph2D
          ref={fgRef}
          graphData={{ nodes, links } as never}
          nodeId="id"
          nodeVal="val"
          nodeColor="color"
          nodeLabel="name"
          linkColor={() => "rgba(148, 163, 184, 0.25)"}
          linkWidth={1.5}
          linkDirectionalArrowLength={0}
          linkLabel={() => ""}
          nodeCanvasObject={(node: GraphNode, ctx, globalScale) => {
            const label = node.name;
            const fontSize = 11 / globalScale;
            const charFaction = charFactionMap.get(node.id);
            ctx.font = `600 ${fontSize}px system-ui, sans-serif`;

            const nodeSize = node.val * 1.8;
            const nodeX = node.x!;
            const nodeY = node.y!;

            ctx.beginPath();
            ctx.arc(nodeX, nodeY, nodeSize, 0, 2 * Math.PI);
            ctx.fillStyle = node.color;
            ctx.globalAlpha = 0.85;
            ctx.fill();
            ctx.globalAlpha = 1;

            ctx.beginPath();
            ctx.arc(nodeX, nodeY, nodeSize + 1.5, 0, 2 * Math.PI);
            ctx.strokeStyle = node.color;
            ctx.lineWidth = 1.5;
            ctx.stroke();

            ctx.fillStyle = "#FFFFFF";
            ctx.textAlign = "center";
            ctx.textBaseline = "middle";
            ctx.fillText(label.charAt(0), nodeX, nodeY);

            const labelY = nodeY + nodeSize + fontSize + 1;
            ctx.font = `500 ${fontSize * 0.9}px system-ui, sans-serif`;
            ctx.fillStyle = "rgba(203, 213, 225, 0.9)";
            ctx.fillText(label, nodeX, labelY);

            if (charFaction?.faction) {
              ctx.font = `400 ${fontSize * 0.75}px system-ui, sans-serif`;
              ctx.fillStyle = "rgba(148, 163, 184, 0.6)";
              ctx.fillText(charFaction.faction, nodeX, labelY + fontSize * 0.9);
            }

            return undefined;
          }}
          linkCanvasObjectMode={() => "after"}
          linkCanvasObject={(link: GraphLink, ctx, globalScale) => {
            const fontSize = 9 / globalScale;
            if (!link.relationshipType && !link.description) return;

            const src = nodes.find((n) => n.id === link.source);
            const tgt = nodes.find((n) => n.id === link.target);
            if (!src || !tgt) return;

            const midX = ((src.x ?? 0) + (tgt.x ?? 0)) / 2;
            const midY = ((src.y ?? 0) + (tgt.y ?? 0)) / 2;

            const text = link.relationshipType || link.description || "";
            ctx.font = `400 ${fontSize}px system-ui, sans-serif`;
            const textWidth = ctx.measureText(text).width;

            ctx.fillStyle = "rgba(15, 23, 42, 0.8)";
            const padding = 3 / globalScale;
            ctx.beginPath();
            ctx.roundRect(
              midX - textWidth / 2 - padding,
              midY - fontSize / 2 - padding,
              textWidth + padding * 2,
              fontSize + padding * 2,
              3 / globalScale,
            );
            ctx.fill();

            ctx.fillStyle = "#e2e8f0";
            ctx.textAlign = "center";
            ctx.textBaseline = "middle";
            ctx.fillText(text, midX, midY);

            return undefined;
          }}
          onNodeClick={handleNodeClick}
          onNodeHover={setHoveredNode}
          onLinkHover={setHoveredLink}
          enableNodeDrag={true}
          enableZoomInteraction={true}
          enablePanInteraction={true}
          warmupTicks={30}
          cooldownTicks={1}
          d3AlphaDecay={0.03}
          d3VelocityDecay={0.3}
          backgroundColor="transparent"
        />

        {hoveredNode && (
          <div
            className={cn(
              "pointer-events-none absolute z-10 rounded-lg border border-border bg-background/95 backdrop-blur-sm px-3 py-2 shadow-lg",
            )}
            style={{
              left: "12px",
              top: "12px",
            }}
          >
            <p className="text-sm font-semibold text-foreground">{hoveredNode.name}</p>
            {hoveredNode.race && (
              <p className="text-xs text-muted-foreground mt-0.5">
                种族：{hoveredNode.race}
              </p>
            )}
            {hoveredNode.faction && (
              <p className="text-xs text-muted-foreground mt-0.5 flex items-center gap-1">
                <Swords className="size-3" />
                {hoveredNode.faction}
                {hoveredNode.factionRole && hoveredNode.factionRole !== "成员" && ` · ${hoveredNode.factionRole}`}
              </p>
            )}
          </div>
        )}

        {hoveredLink && (
          <div
            className={cn(
              "pointer-events-none absolute z-10 rounded-lg border border-border bg-background/95 backdrop-blur-sm px-3 py-2 shadow-lg",
            )}
            style={{
              right: "12px",
              top: "12px",
            }}
          >
            <p className="text-sm font-semibold text-foreground">
              {hoveredLink.relationshipType || "关系"}
            </p>
            {hoveredLink.description && (
              <p className="text-xs text-muted-foreground mt-0.5">{hoveredLink.description}</p>
            )}
          </div>
        )}

        {nodes.length === 0 && (
          <div className="absolute inset-0 flex items-center justify-center">
            <p className="text-sm text-muted-foreground/50">创建角色后关系图将在此显示</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default CharacterGraph;
