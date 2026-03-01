<script lang="ts">
    let { key, schema, value = $bindable() } = $props();

    const title = $derived(key.replace(/_/g, ' '));
    const description = $derived(schema.description || '');
    const type = $derived(schema.type || 'string');
</script>

<div class="form-control w-full">
    <label class="label pb-1" for="field-{key}">
        <span class="label-text text-[10px] font-black uppercase tracking-widest opacity-50">{title}</span>
    </label>
    
    {#if type === 'number'}
        <input 
            type="number" id="field-{key}" 
            bind:value 
            class="input input-bordered input-sm font-mono focus:input-primary transition-all" 
        />
    {:else if type === 'boolean'}
        <div class="flex items-center gap-3 h-8">
            <input 
                type="checkbox" id="field-{key}" 
                bind:checked={value} 
                class="toggle toggle-primary toggle-sm" 
            />
            {#if value !== undefined}
                <span class="text-[10px] font-mono opacity-40 uppercase">{value ? 'ON' : 'OFF'}</span>
            {/if}
        </div>
    {:else if key.includes('password') || key.includes('token') || key.includes('key')}
        <input 
            type="password" id="field-{key}" 
            bind:value 
            class="input input-bordered input-sm font-mono focus:input-primary transition-all" 
        />
    {:else}
        <input 
            type="text" id="field-{key}" 
            bind:value 
            placeholder={description}
            class="input input-bordered input-sm font-mono focus:input-primary transition-all" 
        />
    {/if}

    {#if description}
        <label class="label pt-1" for="field-{key}">
            <span class="label-text-alt text-[9px] opacity-30 italic">{description}</span>
        </label>
    {/if}
</div>
