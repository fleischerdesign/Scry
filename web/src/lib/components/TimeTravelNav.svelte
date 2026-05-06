<script lang="ts">
 import Icon from "@iconify/svelte";

 interface Props {
  groups: [string, any[]][]; // Array of [dateString, events[]]
 }

 let { groups }: Props = $props();

 // Find the current year or the year of the latest event
 const activeYear = $derived.by(() => {
  if (groups.length === 0) return new Date().getFullYear();
  return new Date(groups[0][0]).getFullYear();
 });

 // Create a list of all 12 months for the active year
 const allMonths = $derived.by(() => {
  const months = [];
  const monthNames = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
  
  // Find which months actually have data
  const dataMonths = new Set<number>();
  groups.forEach(([date]) => {
   dataMonths.add(new Date(date).getMonth());
  });

  for (let i = 0; i < 12; i++) {
   months.push({
    index: i,
    label: monthNames[i],
    hasData: dataMonths.has(i),
    // Find the first date string for this month to scroll to
    targetDate: groups.find(([date]) => new Date(date).getMonth() === i)?.[0]
   });
  }
  return months;
 });

 function scrollToDate(date?: string) {
  if (!date) return;
  const id = `group-${date}`;
  const element = document.getElementById(id);
  if (element) {
   const offset = 100; // Professional offset for sticky headers
   const bodyRect = document.body.getBoundingClientRect().top;
   const elementRect = element.getBoundingClientRect().top;
   const elementPosition = elementRect - bodyRect;
   const offsetPosition = elementPosition - offset;

   window.scrollTo({
    top: offsetPosition,
    behavior: 'smooth'
   });
  }
 }
</script>

<aside class="w-32 hidden lg:flex flex-col sticky top-24 h-fit pr-4 border-r border-base-300 mr-8">
 <div class="flex flex-col gap-1">
  <div class="flex items-center gap-2 mb-4 px-2">
   <Icon icon="lucide:calendar" class="w-3 h-3 opacity-60" />
   <span class="text-xs font-bold tracking-wide opacity-60 ">{activeYear}</span>
  </div>

  {#each allMonths as month}
   <button 
    onclick={() => scrollToDate(month.targetDate)}
    disabled={!month.hasData}
    class="flex items-center justify-between px-3 py-2 rounded-xl transition-all duration-300 group
      {month.hasData ? 'hover:bg-primary/10 cursor-pointer' : 'cursor-default opacity-10'}"
   >
    <span class="text-xs font-mono font-bold tracking-wide {month.hasData ? 'text-base-content opacity-60 group-hover:text-primary group-hover:opacity-100' : ''}">
     {month.label}
    </span>
    
    {#if month.hasData}
     <div class="w-1 h-1 rounded-full bg-primary opacity-70 group-hover:scale-150 transition-transform"></div>
    {/if}
   </button>
  {/each}
 </div>
</aside>
