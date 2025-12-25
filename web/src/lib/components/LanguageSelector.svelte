<script lang="ts">
  import type { ProgrammingLanguage } from '$lib/types';
  
  interface Props {
    languages: ProgrammingLanguage[];
    selected: ProgrammingLanguage[];
    onToggle: (lang: ProgrammingLanguage) => void;
  }
  
  let { languages, selected, onToggle }: Props = $props();
</script>

<div class="form-group">
  <label>Supported Languages</label>
  <div class="checkbox-group">
    {#each languages as lang}
      <label class="checkbox-label">
        <input
          type="checkbox"
          checked={selected.includes(lang)}
          onchange={() => onToggle(lang)}
        />
        {lang.toUpperCase()}
      </label>
    {/each}
  </div>
</div>

<style>
  .form-group {
    margin-bottom: 1rem;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
  }
  
  .checkbox-group {
    display: flex;
    gap: 1rem;
  }
  
  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: normal;
  }
</style>
