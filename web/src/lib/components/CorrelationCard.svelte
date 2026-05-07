<script lang="ts">
 let { baseVal, joins, sampleSize, baseLabel, joinLabel, joinUnit } = $props<{
  baseVal: string;
  joins: Record<string, number>;
  sampleSize: number;
  baseLabel?: string;
  joinLabel?: string;
  joinUnit?: string;
 }>();
 
 /** Extract a human-readable label from a raw correlation value (may be JSON or plain string). */
 function displayValue(raw: string): string {
  try {
   const parsed = JSON.parse(raw);
   if (typeof parsed === 'object' && parsed !== null) {
    // Use display_title if available, otherwise first string value, otherwise stringify
    return parsed.display_title
     ?? Object.values(parsed).find((v): v is string => typeof v === 'string')
     ?? JSON.stringify(parsed);
   }
   return String(parsed);
  } catch {
   return raw;
  }
 }
</script>

<div class="card bg-base-100 border border-base-300 shadow-sm hover:shadow-md transition-shadow">
 <div class="card-body p-6">
  <div class="flex items-center gap-3 mb-6">
   <div class="w-1 h-8 bg-secondary rounded-full"></div>
   <div>
    <h3 class="text-xs font-black opacity-70 tracking-tighter">{baseLabel ?? 'Context Cluster'}</h3>
    <p class="text-sm font-bold font-mono truncate max-w-xs text-secondary">
     {displayValue(baseVal)}
    </p>
   </div>
  </div>
  
  <div class="space-y-5">
   {#each Object.entries(joins) as [joinVal, count]}
    <div class="space-y-2">
     <div class="flex justify-between items-end">
      <span class="text-xs font-medium font-mono text-base-content/70">
       {displayValue(joinVal)}{joinUnit ? ` ${joinUnit}` : ''}
      </span>
      <span class="text-xs font-bold opacity-60">{count} occurrences</span>
     </div>
     <progress 
      class="progress progress-secondary w-full h-1.5 opacity-60 hover:opacity-100 transition-opacity" 
       value={Number(count)} 
       max={Number(sampleSize)}
     ></progress>
    </div>
   {/each}
  </div>
 </div>
</div>
