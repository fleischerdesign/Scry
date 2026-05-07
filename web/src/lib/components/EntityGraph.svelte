<script lang="ts">
 import { router } from "../router.svelte";
 import Icon from "@iconify/svelte";
 import { api } from "../api";

 interface Node {
  id: string;
  ns: string;
  typ: string;
  title: string;
  image?: string;
  icon?: string;
  x: number;
  y: number;
 }

 let { 
  relationships, 
  centralEntityId, 
  centralTitle,
  centralImage,
  centralIcon
 }: { 
  relationships: any[], 
  centralEntityId: string, 
  centralTitle: string,
  centralImage?: string | null,
  centralIcon?: string | null
 } = $props();

 let containerWidth = $state(600);
 let containerHeight = $state(400);

 // --- Pan & Zoom State ---
 let scale = $state(1);
 let translateX = $state(0);
 let translateY = $state(0);
 let isDragging = $state(false);
 let startX = 0;
 let startY = 0;

 // Resolved display info for neighbors
 let neighborInfo = $state<Record<string, any>>({});

 const centerX = $derived(containerWidth / 2);
 const centerY = $derived(containerHeight / 2);
 const radius = 160;

 async function resolveNeighbors() {
  const refs = relationships.map(rel => {
   const isSource = rel.source.id === centralEntityId;
   const target = isSource ? rel.target : rel.source;
   return { namespace: target.ns, typ: target.typ, id: target.id };
  });
  
  if (refs.length === 0) return;
  try {
   const resolved = await api.resolveEntities(refs);
   const info: Record<string, any> = {};
   resolved.forEach(ent => { info[ent.id] = ent; });
   neighborInfo = info;
  } catch (e) {
   console.error("Failed to resolve graph neighbors", e);
  }
 }

 $effect(() => {
  if (relationships.length >= 0) resolveNeighbors();
 });

 const nodes = $derived.by(() => {
  const nodeList: Node[] = [];
  nodeList.push({
   id: centralEntityId, ns: "", typ: "",
   title: centralTitle, image: centralImage || undefined, icon: centralIcon || undefined,
   x: centerX, y: centerY
  });

  relationships.forEach((rel, i) => {
   const isSource = rel.source.id === centralEntityId;
   const target = isSource ? rel.target : rel.source;
   const angle = (i / relationships.length) * 2 * Math.PI;
   const x = centerX + radius * Math.cos(angle);
   const y = centerY + radius * Math.sin(angle);
   const info = neighborInfo[target.id];

   nodeList.push({
    id: target.id, ns: target.ns, typ: target.typ,
    title: info?.display_title || target.id,
    image: info?.display_image, icon: info?.display_icon,
    x, y
   });
  });
  return nodeList;
 });

 const edges = $derived.by(() => {
  return relationships.map((rel, i) => {
   const angle = (i / relationships.length) * 2 * Math.PI;
   return {
    label: rel.display_label || rel.predicate.split('/').pop(),
    x: centerX + (radius * 0.55) * Math.cos(angle),
    y: centerY + (radius * 0.55) * Math.sin(angle)
   };
  });
 });

 // --- Interaction Handlers ---
 function handleWheel(e: WheelEvent) {
  e.preventDefault();
  const zoomSpeed = 0.001;
  const delta = -e.deltaY * zoomSpeed;
  const newScale = Math.min(Math.max(0.2, scale + delta), 3);
  
  // Zoom towards mouse position
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const mouseX = e.clientX - rect.left;
  const mouseY = e.clientY - rect.top;

  translateX -= (mouseX - translateX) * (newScale / scale - 1);
  translateY -= (mouseY - translateY) * (newScale / scale - 1);
  scale = newScale;
 }

 function handleMouseDown(e: MouseEvent) {
  if (e.button !== 0) return; // Only left click
  isDragging = true;
  startX = e.clientX - translateX;
  startY = e.clientY - translateY;
 }

 function handleMouseMove(e: MouseEvent) {
  if (!isDragging) return;
  translateX = e.clientX - startX;
  translateY = e.clientY - startY;
 }

 function handleMouseUp() {
  isDragging = false;
 }

 function resetView() {
  scale = 1;
  translateX = 0;
  translateY = 0;
 }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
 class="w-full bg-base-200/30 rounded-3xl border border-base-300 relative overflow-hidden h-[500px] cursor-grab active:cursor-grabbing" 
 bind:clientWidth={containerWidth} 
 bind:clientHeight={containerHeight}
 onwheel={handleWheel}
 onmousedown={handleMouseDown}
 onmousemove={handleMouseMove}
 onmouseup={handleMouseUp}
 onmouseleave={handleMouseUp}
>
 <!-- Background Grid (Static) -->
 <div class="absolute inset-0 opacity-[0.03] pointer-events-none" style="background-image: linear-gradient(#ccc 1px, transparent 1px), linear-gradient(90deg, #ccc 1px, transparent 1px); background-size: 50px 50px; transform: translate({translateX}px, {translateY}px) scale({scale});"></div>

 <svg width={containerWidth} height={containerHeight} class="relative z-10">
  <g transform="translate({translateX}, {translateY}) scale({scale})">
   <!-- Connection Lines -->
   {#each nodes.slice(1) as node, i}
    <line x1={centerX} y1={centerY} x2={node.x} y2={node.y} stroke="currentColor" stroke-width="2" class="text-base-content/10" />
    {@const edge = edges[i]}
    <g transform="translate({edge.x}, {edge.y})">
     <rect x="-35" y="-9" width="70" height="18" rx="9" class="fill-base-100 stroke-base-300" stroke-width="1" />
     <text text-anchor="middle" dy="4" class="fill-base-content/50 font-bold text-xs tracking-tighter">{edge.label}</text>
    </g>
   {/each}

   <!-- Neighbor Nodes -->
   {#each nodes.slice(1) as node}
     <g class="cursor-pointer group" role="button" tabindex="0" onclick={(e) => { e.stopPropagation(); router.navigate(`/entity/${node.ns}/${node.typ}/${node.id}`); }} onkeydown={(e) => e.key === 'Enter' && router.navigate(`/entity/${node.ns}/${node.typ}/${node.id}`)}>
     <circle cx={node.x} cy={node.y} r="28" class="fill-primary/0 group-hover:fill-primary/10 transition-all duration-300" />
     <circle cx={node.x} cy={node.y} r="22" class="fill-base-100 stroke-base-300 group-hover:stroke-primary transition-all shadow-xl" stroke-width="2" />
     {#if node.image}
      <defs><clipPath id="clip-{node.id}"><circle cx={node.x} cy={node.y} r="20" /></clipPath></defs>
      <image href={node.image} x={node.x - 20} y={node.y - 20} width="40" height="40" clip-path="url(#clip-{node.id})" preserveAspectRatio="xMidYMid slice" />
     {:else}
      <foreignObject x={node.x - 10} y={node.y - 10} width="20" height="20">
       <div class="text-base-content/30 group-hover:text-primary transition-colors flex items-center justify-center">
        <Icon icon={node.icon || 'lucide:circle'} class="w-5 h-5" />
       </div>
      </foreignObject>
     {/if}
     <g class="opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none">
      <rect x={node.x - 50} y={node.y + 30} width="100" height="16" rx="6" class="fill-base-800/90" />
      <text x={node.x} y={node.y + 41} text-anchor="middle" class="fill-white font-bold text-xs tracking-wide">{node.title}</text>
     </g>
    </g>
   {/each}

   <!-- Central Node -->
   <g class="filter drop-shadow-2xl">
    <circle cx={centerX} cy={centerY} r="42" class="fill-primary stroke-primary-content/20" stroke-width="6" />
    {#if centralImage}
     <defs><clipPath id="clip-central"><circle cx={centerX} cy={centerY} r="38" /></clipPath></defs>
     <image href={centralImage} x={centerX - 38} y={centerY - 38} width="76" height="76" clip-path="url(#clip-central)" preserveAspectRatio="xMidYMid slice" />
    {:else if centralIcon}
     <foreignObject x={centerX - 20} y={centerY - 20} width="40" height="40">
      <div class="text-primary-content flex items-center justify-center"><Icon icon={centralIcon} class="w-10 h-10" /></div>
     </foreignObject>
    {:else}
     <text x={centerX} y={centerY} dy="6" text-anchor="middle" class="fill-primary-content font-bold text-xl tracking-tighter">{centralTitle.charAt(0).toUpperCase()}</text>
    {/if}
    <circle cx={centerX} cy={centerY} r="48" class="fill-none stroke-primary/20 animate-pulse" stroke-width="1" />
   </g>
  </g>
 </svg>

 <!-- Floating Controls -->
 <div class="absolute top-6 left-6 flex flex-col gap-2">
  <button class="btn btn-sm btn-square btn-ghost bg-base-100/50 backdrop-blur border border-base-300 rounded-xl" onclick={(e) => { e.stopPropagation(); resetView(); }} title="Reset View">
   <Icon icon="lucide:refresh-ccw" class="w-4 h-4 opacity-60" />
  </button>
 </div>

 <!-- Legend Hint -->
 <div class="absolute bottom-8 left-10 flex flex-col gap-2 opacity-70 pointer-events-none">
  <div class="flex items-center gap-2">
   <div class="w-2.5 h-2.5 rounded-full bg-primary shadow-sm shadow-primary/50"></div>
   <span class="text-xs font-bold tracking-wide text-base-content">Active Focus</span>
  </div>
  <div class="flex items-center gap-2">
   <div class="w-2.5 h-2.5 rounded-full bg-base-300 border border-base-content/10"></div>
   <span class="text-xs font-bold tracking-wide text-base-content">Connected Node</span>
  </div>
 </div>

 <!-- Navigation Prompt -->
 <div class="absolute top-8 right-10 opacity-60 text-right pointer-events-none">
  <p class="text-xs font-bold tracking-wide">Graph Explorer</p>
  <p class="text-xs opacity-60 mt-1">Scroll to Zoom · Drag to Pan · Click to Navigate</p>
 </div>
</div>
