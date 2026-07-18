<script lang="ts">
  import { hierarchy, treemap, type HierarchyRectangularNode } from "d3-hierarchy";
  import { num } from "../lib/format";

  interface Leaf {
    name: string;
    value: number;
    group: string;
    title: string;
    color: string;
  }

  let { leaves, height = 420 }: { leaves: Leaf[]; height?: number } = $props();

  let width = $state(900);
  let el: HTMLDivElement;

  $effect(() => {
    const ro = new ResizeObserver((entries) => {
      width = Math.max(320, entries[0].contentRect.width);
    });
    ro.observe(el);
    return () => ro.disconnect();
  });

  interface TreeDatum {
    name: string;
    color?: string;
    title?: string;
    value?: number;
    children?: TreeDatum[];
  }

  const layout = $derived.by(() => {
    const byGroup = new Map<string, Leaf[]>();
    for (const l of leaves) {
      const arr = byGroup.get(l.group) ?? [];
      arr.push(l);
      byGroup.set(l.group, arr);
    }
    const root: TreeDatum = {
      name: "root",
      children: [...byGroup.entries()].map(([group, ls]) => ({
        name: group,
        color: ls[0]?.color,
        children: ls.map((l) => ({
          name: l.name,
          value: l.value,
          title: l.title,
          color: l.color,
        })),
      })),
    };
    const h = hierarchy<TreeDatum>(root)
      .sum((d) => d.value ?? 0)
      .sort((a, b) => (b.value ?? 0) - (a.value ?? 0));
    treemap<TreeDatum>().size([width, height]).paddingInner(1).paddingTop(0).round(true)(h);
    return h as HierarchyRectangularNode<TreeDatum>;
  });

  const cells = $derived(layout.leaves());
</script>

<div class="tm" bind:this={el}>
  <svg {width} {height} role="img" aria-label="Model composition treemap">
    {#each cells as c (c.data.name + c.x0 + c.y0)}
      {@const w = c.x1 - c.x0}
      {@const h = c.y1 - c.y0}
      <g transform="translate({c.x0},{c.y0})">
        <rect
          width={w}
          height={h}
          rx="2"
          fill={c.data.color ?? "var(--accent)"}
          fill-opacity="0.85"
        >
          <title>{c.data.title ?? c.data.name}: {num(c.value ?? 0)}</title>
        </rect>
        {#if w > 46 && h > 20}
          <text x="4" y="14" font-size="11" fill="#fff" style="paint-order:stroke; pointer-events:none">
            {c.data.name}
          </text>
        {/if}
      </g>
    {/each}
  </svg>
</div>

<style>
  .tm {
    width: 100%;
  }
  svg {
    display: block;
    max-width: 100%;
  }
</style>
