<script lang="ts">
  import type { ProgrammingLanguage } from '$lib/types';
  
  interface Props {
    languages: ProgrammingLanguage[];
    selected: ProgrammingLanguage[];
    onToggle: (lang: ProgrammingLanguage) => void;
  }
  
  let { languages, selected, onToggle }: Props = $props();
</script>

<fieldset class="form-group">
  <legend>Supported Languages</legend>
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
</fieldset>

<style>
  .form-group {
    margin-bottom: 1rem;
    border: none;
    padding: 0;
  }

  .form-group legend {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    padding: 0;
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
