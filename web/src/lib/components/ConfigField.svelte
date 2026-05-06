<script lang="ts">
 let { key, schema, value = $bindable() } = $props();

 const title = $derived(key.replace(/_/g, ' '));
 const description = $derived(schema.description || '');
 const type = $derived(schema.type || 'string');
 const isSecret = $derived(schema.secret === true || key.includes('password') || key.includes('token') || key.includes('key'));
 const isRequired = $derived(schema.required === true);
 const isEmpty = $derived(value === undefined || value === null || value === '');
 const hasError = $derived(isRequired && isEmpty);
 let showSecret = $state(false);
 let touched = $state(false);
</script>

<div class="form-control w-full">
 <label class="label pb-1" for="field-{key}">
  <span class="label-text text-xs font-bold tracking-wide opacity-70">
   {title}
   {#if isRequired}
    <span class="text-error ml-0.5">*</span>
   {/if}
  </span>
  {#if isSecret}
   <span class="badge badge-warning badge-xs font-mono ml-2">SECRET</span>
  {/if}
 </label>
 
 {#if type === 'number'}
  <input 
   type="number" id="field-{key}" 
   bind:value 
   class="input input-bordered input-sm font-mono focus:input-primary transition-all {hasError ? 'input-error' : ''}"
   oninput={() => touched = true}
  />
 {:else if type === 'boolean'}
  <div class="flex items-center gap-3 h-8">
   <input 
    type="checkbox" id="field-{key}" 
    bind:checked={value} 
    class="toggle toggle-primary toggle-sm" 
   />
   {#if value !== undefined}
    <span class="text-xs font-mono opacity-70 ">{value ? 'ON' : 'OFF'}</span>
   {/if}
  </div>
 {:else if isSecret}
  <div class="relative">
   <input 
    type={showSecret ? "text" : "password"} id="field-{key}" 
    bind:value 
    placeholder={description}
    class="input input-bordered input-sm font-mono focus:input-primary transition-all w-full pr-10 {hasError ? 'input-error' : ''}"
    oninput={() => touched = true}
   />
   <button 
    type="button"
    class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs"
    onclick={() => showSecret = !showSecret}
    aria-label={showSecret ? "Hide secret" : "Show secret"}
   >
    {#if showSecret}
     <span class="lucide lucide-eye-off">👁</span>
    {:else}
     <span class="lucide lucide-eye">👁</span>
    {/if}
   </button>
  </div>
 {:else}
  <input 
   type="text" id="field-{key}" 
   bind:value 
   placeholder={description}
   class="input input-bordered input-sm font-mono focus:input-primary transition-all {hasError ? 'input-error' : ''}"
   oninput={() => touched = true}
  />
 {/if}

 {#if touched && hasError}
  <span class="label-text-alt text-error text-xs mt-1">This field is required</span>
 {:else if description}
  <label class="label pt-1" for="field-{key}">
   <span class="label-text-alt text-xs opacity-60 italic">{description}</span>
  </label>
 {/if}
</div>
