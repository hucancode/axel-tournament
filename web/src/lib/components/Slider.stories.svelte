<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Slider from './Slider.svelte';

  const { Story } = defineMeta({
    title: 'Data Entry/Slider',
    component: Slider,
  });
</script>

<Story name="Default" args={{ value: 50 }} />

<Story name="With Label" args={{ value: 75, label: 'Volume' }} />

<Story name="Without Value Display" args={{ value: 30, label: 'Brightness', showValue: false }} />

<Story name="Custom Range" args={{ value: 5, min: 0, max: 10, label: 'Rating' }} />

<Story name="With Step" args={{ value: 50, min: 0, max: 100, step: 10, label: 'Percentage' }} />

<Story name="Disabled" args={{ value: 60, label: 'Disabled Slider', disabled: true }} />

<Story name="Interactive">
  {#snippet template(args)}
    <script lang="ts">
      let sliderValue = $state(args.value ?? 50);

      function handleValueChange(value: number) {
        sliderValue = value;
        console.log('Slider value changed to:', value);
      }
    </script>

    <div class="flex flex-col gap-4">
      <div class="text-center">
        <p class="mb-2">Current Value: <strong>{args.value}</strong></p>
      </div>
      <Slider
        value={args.value}
        min={0}
        max={100}
        step={1}
        label="Adjust Value"
      />
    </div>
  {/snippet}
</Story>

<Story name="All Variants">
  <div class="flex flex-col gap-6">
    <div>
      <p class="mb-2 font-semibold">Basic Slider</p>
      <Slider value={50} />
    </div>

    <div>
      <p class="mb-2 font-semibold">With Label and Value</p>
      <Slider value={75} label="Volume" />
    </div>

    <div>
      <p class="mb-2 font-semibold">Without Value Display</p>
      <Slider value={30} label="Brightness" showValue={false} />
    </div>

    <div>
      <p class="mb-2 font-semibold">Custom Range (0-10)</p>
      <Slider value={5} min={0} max={10} label="Rating" />
    </div>

    <div>
      <p class="mb-2 font-semibold">With Step (10)</p>
      <Slider value={50} min={0} max={100} step={10} label="Percentage" />
    </div>

    <div>
      <p class="mb-2 font-semibold">Disabled State</p>
      <Slider value={60} label="Disabled" disabled={true} />
    </div>
  </div>
</Story>
