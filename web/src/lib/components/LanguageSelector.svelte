<script lang="ts">
  import type { ProgrammingLanguage } from '$lib/types';
  
  interface Props {
    languages: ProgrammingLanguage[];
    selected: ProgrammingLanguage[];
    onToggle: (lang: ProgrammingLanguage) => void;
  }
  
  let { languages, selected, onToggle }: Props = $props();
</script>

<fieldset>
  <legend>Supported Languages</legend>
  <div>
    {#each languages as lang}
      <label>
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
  fieldset {
    margin-bottom: 1rem;
    border: none;
    padding: 0;
  }

  legend {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    padding: 0;
  }

  fieldset > div {
    display: flex;
    gap: 1rem;
  }

  label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: normal;
  }
</style>
