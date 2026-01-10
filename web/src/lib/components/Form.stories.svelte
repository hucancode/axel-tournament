<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';

  const { Story } = defineMeta({
    title: 'Data Entry/Form',
  });
</script>

<Story name="All Elements">
  {#snippet template()}
    <form>
      <fieldset>
        <legend>Text Inputs</legend>
        <label for="text">Text Input</label>
        <input type="text" id="text" placeholder="Enter text" />

        <label for="email">Email Input</label>
        <input type="email" id="email" placeholder="Enter email" />

        <label for="password">Password Input</label>
        <input type="password" id="password" placeholder="Enter password" />
      </fieldset>

      <fieldset>
        <legend>Select & Range</legend>
        <label for="select">Select Dropdown</label>
        <select id="select">
          <option value="">Choose option</option>
          <option value="1">Option 1</option>
          <option value="2">Option 2</option>
          <option value="3">Option 3</option>
        </select>

        <label for="range">Range Slider</label>
        <input type="range" id="range" min="0" max="100" value="50" />
      </fieldset>

      <fieldset>
        <legend>Textarea & Progress</legend>
        <label for="textarea">Textarea</label>
        <textarea id="textarea" placeholder="Enter long text"></textarea>

        <label for="progress">Progress Bar</label>
        <progress id="progress" value="70" max="100">70%</progress>
      </fieldset>

      <fieldset>
        <legend>Buttons</legend>
        <button type="button">Default Button</button>
        <button type="button" data-variant="primary">Primary Button</button>
        <button type="button" data-variant="secondary">Secondary Button</button>
        <button type="button" data-variant="success">Success Button</button>
        <button type="button" data-variant="danger">Danger Button</button>
        <button type="button" data-variant="ghost">Ghost Button</button>
        <button type="button" disabled>Disabled Button</button>
      </fieldset>
    </form>
  {/snippet}
</Story>

<style>
  fieldset {
    border: 1px solid var(--color-border-strong);
    margin-bottom: 2rem;
    padding: 1rem;
  }

  legend {
    font-weight: 600;
    padding: 0 0.5rem;
  }

  label {
    display: block;
    margin: 1rem 0 0.5rem 0;
    font-weight: 500;
  }

  input, select, textarea {
    margin-bottom: 1rem;
  }

  button {
    margin-right: 0.5rem;
    margin-bottom: 0.5rem;
  }
</style>
