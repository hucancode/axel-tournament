<script lang="ts">
  import type { ProgrammingLanguage } from '$lib/types';
  
  interface Props {
    languages: ProgrammingLanguage[];
    selected: ProgrammingLanguage[];
    onToggle: (lang: ProgrammingLanguage) => void;
  }
  
  let { languages, selected, onToggle }: Props = $props();
</script>

<fieldset class="mb-4 border-0 p-0">
  <legend class="block mb-2 font-medium p-0">Supported Languages</legend>
  <div class="flex gap-4">
    {#each languages as lang}
      <label class="flex items-center gap-2 font-normal">
        <input
          type="checkbox"
          checked={selected.includes(lang)}
          onchange={() => onToggle(lang)}
        />
        {lang.toUpperCase()}
      </label>
    {/each}
  </div>
</fieldset>
